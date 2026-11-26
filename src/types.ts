export type ProviderId = string;
export type AuthState = "unknown" | "authenticated" | "logged_out";
export type ThemeMode = "system" | "light" | "dark";

export interface AccountKey {
  providerId: ProviderId;
  uuid: string;
}

export interface ProviderInfo {
  id: ProviderId;
  name: string;
  shortName: string;
  enabled: boolean;
  isCustom?: boolean;
  statusText: string;
  entryUrl: string;
  homepageUrl: string;
  callbackOrigins: string[];
  verifiedMethods: string[];
}

export interface AppSettings {
  minimizeToTray: boolean;
  globalShortcut: string;
  autoApprove: boolean;
}

export interface ProxyTestResult {
  ok: boolean;
  latencyMs: number;
  statusCode: number | null;
  targetHost: string;
  usedProxy: boolean;
  message: string;
}

export interface QuotaDetail {
  planLabel: string;
  planValue: string;
  planSubtext: string | null;
  planUsedPercent: number | null;
  extraLabel: string | null;
  extraValue: string | null;
  extraSubtext: string | null;
  extraUsedPercent: number | null;
}

export interface AccountRecord {
  providerId: ProviderId;
  uuid: string;
  alias: string;
  createdAtMs: number;
  autoAlias: boolean;
  proxyUrl: string | null;
  timezone: string | null;
  avatarUrl: string | null;
  quotaText: string | null;
  quotaPercent: number | null;
  quotaDetail: QuotaDetail | null;
  authState: AuthState;
  cookieExpiresAtMs: number | null;
  cookieSessionOnly: boolean;
}

export interface ConflictItem {
  uuid: string;
  providers: ProviderId[];
}

export type GroupReplyStatus =
  | "queued"
  | "sending"
  | "streaming"
  | "done"
  | "needs_auth"
  | "error";

export interface GroupSharedSkill {
  id: string;
  name: string;
  description: string;
  content: string;
  enabled: boolean;
}

export interface GroupMemberRole {
  member: AccountKey;
  role: string;
}

export type GroupWorkflowMode = "parallel" | "review" | "a2a_relay";

export interface GroupThreadBinding {
  member: AccountKey;
  threadUrl: string;
  updatedAtMs: number;
}

export type ExternalAgentKind = "openai_api" | "local_cli";

export interface ExternalAgentRecord {
  id: string;
  kind: ExternalAgentKind;
  name: string;
  baseUrl: string;
  apiKey: string;
  model: string;
  command: string;
  args: string[];
  workdir: string | null;
  createdAtMs: number;
}

export interface SiteAdapterConfig {
  id: string;
  name: string;
  hostPatterns: string[];
  composerSelectors: string[];
  sendButtonSelectors: string[];
  stopButtonSelectors: string[];
  messageListSelectors: string[];
  assistantMessageSelectors: string[];
  newChatSelectors: string[];
  newChatUrl: string | null;
  threadUrlPatterns: string[];
  transientStatusWords: string[];
  chatApiPatterns: string[];
  preferEnterSubmit: boolean;
  asyncAgent?: boolean;
}

export interface SiteAdaptersSnapshot {
  adapters: SiteAdapterConfig[];
  overridePath: string;
  hasOverrides: boolean;
  overrideJson: string;
}

export interface GroupChatMessage {
  id: string;
  role: "user" | "assistant" | string;
  sender: AccountKey | null;
  senderName: string;
  content: string;
  createdAtMs: number;
  status: GroupReplyStatus;
  targets: AccountKey[];
  error: string | null;
  stageLabel?: string | null;
  hopDepth?: number;
  autoRoutedChannel?: string | null;
}

export interface GroupChatRecord {
  id: string;
  name: string;
  members: AccountKey[];
  createdAtMs: number;
  messages: GroupChatMessage[];
  sharedContext: string;
  sharedSkills: GroupSharedSkill[];
  memberRoles: GroupMemberRole[];
  workflowMode?: GroupWorkflowMode;
  maxRelayHops?: number;
  threadBindings?: GroupThreadBinding[];
}

export interface AccountPageError {
  providerId: ProviderId;
  uuid: string;
  message: string;
}

export interface ManagerSnapshot {
  platform: string;
  providers: ProviderInfo[];
  accounts: AccountRecord[];
  selectedProviderId: ProviderId;
  activeAccount: AccountKey | null;
  switchingAccount: AccountKey | null;
  warmAccounts: AccountKey[];
  lastCommitted: AccountKey | null;
  pendingCleanup: AccountKey[];
  conflicts: ConflictItem[];
  notice: string | null;
  modalActive: boolean;
  settings: AppSettings;
  globalShortcutActive: boolean;
  globalShortcutError: string | null;
  groupChats: GroupChatRecord[];
  activeGroupId: string | null;
  externalAgents: ExternalAgentRecord[];
  accountErrors?: AccountPageError[];
}

export interface UpdateCheckResult {
  currentVersion: string;
  latestVersion: string;
  hasUpdate: boolean;
  releaseNotes: string;
  pubDate: string | null;
  downloadUrl: string | null;
  sha256: string | null;
  filename: string | null;
  size: number | null;
  releasePageUrl: string;
}

export interface UpdateProgressEvent {
  stage: string;
  downloaded: number;
  total: number | null;
  percent: number | null;
  message: string;
}

export interface InstallUpdateOutcome {
  status: string;
  message: string;
}

export interface DownloadNoticeEvent {
  id: number;
  filename: string;
  status: "started" | "completed" | "failed";
  error?: string | null;
}
