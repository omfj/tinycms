export const themeStorageKey = "tinycms.theme";

export function initialDarkTheme() {
  return window.localStorage.getItem(themeStorageKey) === "dark";
}
