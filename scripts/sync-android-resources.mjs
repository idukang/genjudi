import { cpSync, readFileSync, writeFileSync } from "node:fs";

const source = new URL("../src-tauri/icons/android/", import.meta.url);
const destination = new URL("../src-tauri/gen/android/app/src/main/res/", import.meta.url);
const configPath = new URL("../src-tauri/tauri.conf.json", import.meta.url);
const { productName } = JSON.parse(readFileSync(configPath, "utf8"));
const appLabel = productName.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");

cpSync(source, destination, { recursive: true });

const stringsPath = new URL("values/strings.xml", destination);
const strings = readFileSync(stringsPath, "utf8");
const updatedStrings = strings.replace(
  /(<string name="(?:app_name|main_activity_title)">)[^<]*(<\/string>)/g,
  (_match, openingTag, closingTag) => `${openingTag}"${appLabel}"${closingTag}`,
);

if (updatedStrings !== strings) {
  writeFileSync(stringsPath, updatedStrings);
}
