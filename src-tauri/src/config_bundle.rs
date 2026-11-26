use crate::storage::{
  now_ms, AccountKey, AccountsStoreFile, AppSettings, CustomProviderRecord, ExternalAgentRecord,
  GroupChatRecord, GroupMemberRole, GroupSharedSkill, GroupWorkflowMode,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

pub const BUNDLE_SCHEMA: u32 = 1;
const DEFAULT_RELAY_HOPS: u8 = 2;

/// 群聊在备份中只保留元数据与知识配置（成员、角色、共享上下文与技能），不包含历史消息。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupBackup {
  pub id: String,
  pub name: String,
  #[serde(default)]
  pub members: Vec<AccountKey>,
  #[serde(default)]
  pub shared_context: String,
  #[serde(default)]
  pub shared_skills: Vec<GroupSharedSkill>,
  #[serde(default)]
  pub member_roles: Vec<GroupMemberRole>,
  #[serde(default)]
  pub workflow_mode: GroupWorkflowMode,
  #[serde(default)]
  pub max_relay_hops: Option<u8>,
  #[serde(default)]
  pub created_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundlePayload {
  pub app_version: String,
  pub exported_at_ms: u64,
  #[serde(default)]
  pub custom_providers: Vec<CustomProviderRecord>,
  #[serde(default)]
  pub external_agents: Vec<ExternalAgentRecord>,
  #[serde(default)]
  pub group_chats: Vec<GroupBackup>,
  #[serde(default)]
  pub settings: Option<AppSettings>,
  #[serde(default)]
  pub includes_api_keys: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleImportSummary {
  pub custom_providers_added: usize,
  pub custom_providers_updated: usize,
  pub external_agents_added: usize,
  pub external_agents_updated: usize,
  pub groups_added: usize,
  pub groups_updated: usize,
  pub settings_applied: bool,
  pub includes_api_keys: bool,
  pub checksum: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleImportOutcome {
  pub summary: BundleImportSummary,
  pub snapshot: crate::session_manager::ManagerSnapshot,
}

pub fn sha256_hex(bytes: &[u8]) -> String {
  let mut hasher = Sha256::new();
  hasher.update(bytes);
  let digest = hasher.finalize();
  let mut out = String::with_capacity(digest.len() * 2);
  for byte in digest {
    out.push_str(&format!("{byte:02x}"));
  }
  out
}

fn group_to_backup(group: &GroupChatRecord) -> GroupBackup {
  GroupBackup {
    id: group.id.clone(),
    name: group.name.clone(),
    members: group.members.clone(),
    shared_context: group.shared_context.clone(),
    shared_skills: group.shared_skills.clone(),
    member_roles: group.member_roles.clone(),
    workflow_mode: group.workflow_mode,
    max_relay_hops: Some(group.max_relay_hops),
    created_at_ms: group.created_at_ms,
  }
}

/// 生成可离线备份的配置包 JSON（含 SHA-256 校验和）。
pub fn build_bundle(store: &AccountsStoreFile, include_api_keys: bool) -> Result<String, String> {
  let payload = BundlePayload {
    app_version: env!("CARGO_PKG_VERSION").to_string(),
    exported_at_ms: now_ms(),
    custom_providers: store.custom_providers.clone(),
    external_agents: store
      .external_agents
      .iter()
      .map(|agent| {
        let mut cloned = agent.clone();
        if !include_api_keys {
          cloned.api_key.clear();
        }
        cloned
      })
      .collect(),
    group_chats: store.group_chats.iter().map(group_to_backup).collect(),
    settings: Some(store.settings.clone()),
    includes_api_keys: include_api_keys,
  };

  let payload_json =
    serde_json::to_string(&payload).map_err(|e| format!("序列化备份内容失败: {e}"))?;
  let checksum = sha256_hex(payload_json.as_bytes());
  let payload_value: Value =
    serde_json::from_str(&payload_json).map_err(|e| format!("构造备份结构失败: {e}"))?;
  let bundle = serde_json::json!({
    "schema": BUNDLE_SCHEMA,
    "checksum": checksum,
    "payload": payload_value,
  });
  serde_json::to_string_pretty(&bundle).map_err(|e| format!("输出备份 JSON 失败: {e}"))
}

fn parse_and_verify(raw: &str) -> Result<(BundlePayload, String), String> {
  let trimmed = raw.trim();
  if trimmed.is_empty() {
    return Err("备份内容为空".to_string());
  }
  let root: Value =
    serde_json::from_str(trimmed).map_err(|e| format!("备份不是有效的 JSON: {e}"))?;
  let schema = root.get("schema").and_then(|v| v.as_u64()).unwrap_or(0);
  if schema != BUNDLE_SCHEMA as u64 {
    return Err(format!(
      "备份格式版本不受支持（schema={schema}，本机支持 {BUNDLE_SCHEMA}）"
    ));
  }
  let payload_value = root
    .get("payload")
    .cloned()
    .ok_or_else(|| "备份缺少 payload 字段".to_string())?;
  let declared = root
    .get("checksum")
    .and_then(|v| v.as_str())
    .unwrap_or("")
    .trim()
    .to_ascii_lowercase();
  if declared.is_empty() {
    return Err("备份缺少校验和（checksum），无法验证完整性".to_string());
  }
  let canonical = serde_json::to_string(&payload_value)
    .map_err(|e| format!("读取备份内容失败: {e}"))?;
  let actual = sha256_hex(canonical.as_bytes());
  if actual != declared {
    return Err("备份校验和不匹配，文件可能已损坏或被修改".to_string());
  }
  let payload: BundlePayload =
    serde_json::from_value(payload_value).map_err(|e| format!("备份字段格式无效: {e}"))?;
  Ok((payload, actual))
}

/// 校验并合并备份：自定义服务与 API 成员按 id 覆盖，群聊按 id 合并（保留本机历史消息），设置整体恢复。
pub fn apply_bundle(
  store: &mut AccountsStoreFile,
  raw: &str,
) -> Result<BundleImportSummary, String> {
  let (payload, checksum) = parse_and_verify(raw)?;
  let mut summary = BundleImportSummary {
    checksum,
    includes_api_keys: payload.includes_api_keys,
    ..BundleImportSummary::default()
  };

  for provider in payload.custom_providers {
    let existing_idx = store
      .custom_providers
      .iter()
      .position(|item| item.id == provider.id);
    match existing_idx {
      Some(idx) => {
        store.custom_providers[idx] = provider;
        summary.custom_providers_updated += 1;
      }
      None => {
        store.custom_providers.push(provider);
        summary.custom_providers_added += 1;
      }
    }
  }

  for agent in payload.external_agents {
    let existing_idx = store
      .external_agents
      .iter()
      .position(|item| item.id == agent.id);
    match existing_idx {
      Some(idx) => {
        let keep_existing_key = agent.api_key.trim().is_empty();
        let previous_key = store.external_agents[idx].api_key.clone();
        store.external_agents[idx] = agent;
        if keep_existing_key {
          store.external_agents[idx].api_key = previous_key;
        }
        summary.external_agents_updated += 1;
      }
      None => {
        store.external_agents.push(agent);
        summary.external_agents_added += 1;
      }
    }
  }

  for group in payload.group_chats {
    let clean_id = group.id.trim().to_string();
    if clean_id.is_empty() {
      continue;
    }
    let existing_idx = store.group_chats.iter().position(|item| item.id == clean_id);
    match existing_idx {
      Some(idx) => {
        let existing = &mut store.group_chats[idx];
        if !group.name.trim().is_empty() {
          existing.name = group.name.trim().to_string();
        }
        for member in group.members {
          if !existing.members.contains(&member) {
            existing.members.push(member);
          }
        }
        if !group.shared_context.trim().is_empty() {
          existing.shared_context = group.shared_context;
        }
        for skill in group.shared_skills {
          let skill_idx = existing.shared_skills.iter().position(|item| item.id == skill.id);
          match skill_idx {
            Some(pos) => existing.shared_skills[pos] = skill,
            None => existing.shared_skills.push(skill),
          }
        }
        for role in group.member_roles {
          let role_idx = existing
            .member_roles
            .iter()
            .position(|item| item.member == role.member);
          match role_idx {
            Some(pos) => existing.member_roles[pos] = role,
            None => existing.member_roles.push(role),
          }
        }
        existing.workflow_mode = group.workflow_mode;
        if let Some(hops) = group.max_relay_hops {
          existing.max_relay_hops = hops.clamp(1, 3);
        }
        summary.groups_updated += 1;
      }
      None => {
        store.group_chats.push(GroupChatRecord {
          id: clean_id,
          name: if group.name.trim().is_empty() {
            "导入的群聊".to_string()
          } else {
            group.name.trim().to_string()
          },
          members: group.members,
          created_at_ms: if group.created_at_ms > 0 {
            group.created_at_ms
          } else {
            now_ms()
          },
          messages: Vec::new(),
          shared_context: group.shared_context,
          shared_skills: group.shared_skills,
          member_roles: group.member_roles,
          workflow_mode: group.workflow_mode,
          max_relay_hops: group
            .max_relay_hops
            .map(|hops| hops.clamp(1, 3))
            .unwrap_or(DEFAULT_RELAY_HOPS),
          thread_bindings: Vec::new(),
        });
        summary.groups_added += 1;
      }
    }
  }

  if let Some(settings) = payload.settings {
    store.settings = settings;
    summary.settings_applied = true;
  }

  if summary.custom_providers_added == 0
    && summary.custom_providers_updated == 0
    && summary.external_agents_added == 0
    && summary.external_agents_updated == 0
    && summary.groups_added == 0
    && summary.groups_updated == 0
    && !summary.settings_applied
  {
    return Err("备份中没有任何可恢复的配置内容".to_string());
  }

  Ok(summary)
}

#[cfg(test)]
mod tests {
  use super::{apply_bundle, build_bundle, parse_and_verify};
  use crate::providers::ProviderId;
  use crate::storage::{
    now_ms, AccountKey, AccountsStoreFile, AppSettings, CustomProviderRecord, ExternalAgentKind,
    ExternalAgentRecord, GroupChatMessage, GroupChatRecord, GroupSharedSkill, GroupReplyStatus,
  };

  fn empty_store() -> AccountsStoreFile {
    AccountsStoreFile {
      version: 1,
      accounts: Vec::new(),
      custom_providers: Vec::new(),
      last_committed: None,
      pending_cleanup: Vec::new(),
      group_chats: Vec::new(),
      external_agents: vec![ExternalAgentRecord {
        id: uuid::Uuid::new_v4().hyphenated().to_string(),
        kind: ExternalAgentKind::OpenaiApi,
        name: "测试 API".to_string(),
        base_url: "https://api.example.com/v1".to_string(),
        api_key: "sk-secret".to_string(),
        model: "gpt-test".to_string(),
        command: String::new(),
        args: Vec::new(),
        workdir: None,
        created_at_ms: now_ms(),
      }],
      settings: AppSettings {
        minimize_to_tray: true,
        global_shortcut: "CommandOrControl+Shift+M".to_string(),
        auto_approve: false,
      },
    }
  }

  #[test]
  fn bundle_roundtrip_restores_config_without_messages() {
    let mut source = empty_store();
    let member_key = AccountKey {
      provider_id: ProviderId::api(),
      uuid: source.external_agents[0].id.clone(),
    };
    source.group_chats.push(GroupChatRecord {
      id: "group-1".to_string(),
      name: "研究小组".to_string(),
      members: vec![member_key.clone()],
      created_at_ms: 1,
      messages: vec![GroupChatMessage {
        id: "msg-1".to_string(),
        role: "user".to_string(),
        sender: None,
        sender_name: "我".to_string(),
        content: "历史消息不应被备份".to_string(),
        created_at_ms: 2,
        status: GroupReplyStatus::Done,
        targets: Vec::new(),
        error: None,
        stage_label: None,
        hop_depth: 0,
        auto_routed_channel: None,
      }],
      shared_context: "共享背景".to_string(),
      shared_skills: vec![GroupSharedSkill {
        id: "skill-1".to_string(),
        name: "简洁".to_string(),
        description: String::new(),
        content: "直接给结论".to_string(),
        enabled: true,
      }],
      member_roles: Vec::new(),
      workflow_mode: Default::default(),
      max_relay_hops: 2,
      thread_bindings: Vec::new(),
    });
    source.custom_providers.push(CustomProviderRecord {
      id: ProviderId::parse("custom-demo").expect("provider id"),
      name: "Demo".to_string(),
      entry_url: "https://demo.example.com/".to_string(),
      extra_hosts: Vec::new(),
      created_at_ms: 1,
    });

    let bundle = build_bundle(&source, false).expect("build bundle");
    let (payload, _) = parse_and_verify(&bundle).expect("verify bundle");
    assert_eq!(payload.group_chats.len(), 1);
    assert!(payload.external_agents[0].api_key.is_empty());

    let mut target = empty_store();
    let summary = apply_bundle(&mut target, &bundle).expect("apply bundle");
    assert_eq!(summary.groups_added, 1);
    assert_eq!(summary.custom_providers_added, 1);
    assert_eq!(target.group_chats.len(), 1);
    assert!(target.group_chats[0].messages.is_empty());
    assert_eq!(target.group_chats[0].shared_context, "共享背景");
    assert_eq!(target.group_chats[0].members, vec![member_key]);
    // 不含 API Key 的备份不会覆盖本机已保存的 Key
    assert_eq!(target.external_agents[0].api_key, "sk-secret");
  }

  #[test]
  fn bundle_rejects_tampered_payload() {
    let source = empty_store();
    let bundle = build_bundle(&source, true).expect("build bundle");
    let tampered = bundle.replace("\"测试 API\"", "\"被篡改\"");
    assert!(parse_and_verify(&tampered).is_err());
  }
}
