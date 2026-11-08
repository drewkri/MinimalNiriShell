// This work is all offloaded to the Wifi Commander extension (https://www.vicinae.com/extensions/dagimg-dot/wifi-commander)
// so this menu will just route you to that extension. It must be
// installed for this to work properly.

import { Action, ActionPanel, Icon, List } from "@vicinae/api";
import { JSX } from "react/jsx-runtime";
import { CurrentPreferences, openDeepLink } from "./util";

export const NetworkMenu: JSX.Element = (
  <List searchBarPlaceholder={"Search menu..."}>
    <List.Section title={"Quick Access"}></List.Section>
    <List.Item
      title="Manage Saved Networks"
      icon={Icon.Cog}
      actions={
        <ActionPanel>
          <Action
            title="Manage Saved Networks"
            onAction={() => {
              openDeepLink(
                "vicinae://launch/@dagimg-dot/store.vicinae.wifi-commander/manage-saved-networks",
              );
            }}
          />
        </ActionPanel>
      }
    ></List.Item>
    <List.Item
      title="Scan Wifi Networks"
      icon={Icon.MagnifyingGlass}
      actions={
        <ActionPanel>
          <Action
            title="Scan Wifi Networks"
            onAction={() => {
              openDeepLink(
                "vicinae://launch/@dagimg-dot/store.vicinae.wifi-commander/scan-wifi",
              );
            }}
          />
        </ActionPanel>
      }
    ></List.Item>
    <List.Item
      title="Turn Wifi Off"
      icon={Icon.WifiDisabled}
      actions={
        <ActionPanel>
          <Action
            title="Turn Wifi Off"
            onAction={() => {
              openDeepLink(
                "vicinae://launch/@dagimg-dot/store.vicinae.wifi-commander/toggle-wifi-off",
              );
            }}
          />
        </ActionPanel>
      }
    ></List.Item>
    <List.Item
      title="Turn Wifi On"
      icon={Icon.Wifi}
      actions={
        <ActionPanel>
          <Action
            title="Turn Wifi On"
            onAction={() => {
              openDeepLink(
                "vicinae://launch/@dagimg-dot/store.vicinae.wifi-commander/toggle-wifi-on",
              );
            }}
          />
        </ActionPanel>
      }
    ></List.Item>
    <List.Item
      title="Restart Wifi"
      icon={Icon.Redo}
      actions={
        <ActionPanel>
          <Action
            title="Restart Wifi"
            onAction={() => {
              openDeepLink(
                "vicinae://launch/@dagimg-dot/store.vicinae.wifi-commander/restart-wifi",
              );
            }}
          />
        </ActionPanel>
      }
    ></List.Item>
    <List.Item
      title="Open advanced network settings"
      icon={Icon.WrenchScrewdriver}
      actions={
        <ActionPanel>
          <Action
            title="Open advanced network settings"
            onAction={() => {
              openDeepLink(CurrentPreferences.networkSettingsLink);
            }}
          />
        </ActionPanel>
      }
    ></List.Item>
  </List>
);
