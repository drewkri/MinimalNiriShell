// This work is all offloaded to the PulseAudio extension (https://www.vicinae.com/extensions/rastsislaux/pulseaudio)
// so this menu will just route you to that extension. It must be
// installed for this to work properly.

import { Action, ActionPanel, Icon, List } from "@vicinae/api";
import { JSX } from "react/jsx-runtime";
import { openDeepLink } from "./util";

export const BluetoothMenu: JSX.Element = (
  <List searchBarPlaceholder={"Search menu..."}>
    <List.Section title={"Quick Access"}></List.Section>
    <List.Item
      title="Devices"
      icon={Icon.Devices}
      actions={
        <ActionPanel>
          <Action
            title="Devices"
            onAction={() => {
              openDeepLink(
                "vicinae://launch/@Gelei/store.vicinae.bluetooth/devices",
              );
            }}
          />
        </ActionPanel>
      }
    ></List.Item>
    <List.Item
      title="Scan"
      icon={Icon.MagnifyingGlass}
      actions={
        <ActionPanel>
          <Action
            title="Scan"
            onAction={() => {
              openDeepLink(
                "vicinae://launch/@Gelei/store.vicinae.bluetooth/scan",
              );
            }}
          />
        </ActionPanel>
      }
    ></List.Item>
    <List.Item
      title="Make this device discoverable"
      icon={Icon.Eye}
      actions={
        <ActionPanel>
          <Action
            title="Make this device discoverable"
            onAction={() => {
              openDeepLink(
                "vicinae://launch/@Gelei/store.vicinae.bluetooth/discoverable",
              );
            }}
          />
        </ActionPanel>
      }
    ></List.Item>
    <List.Item
      title="Turn bluetooth on"
      icon={Icon.Bluetooth}
      actions={
        <ActionPanel>
          <Action
            title="Turn bluetooth on"
            onAction={() => {
              openDeepLink(
                "vicinae://launch/@Gelei/store.vicinae.bluetooth/bluetooth-on",
              );
            }}
          />
        </ActionPanel>
      }
    ></List.Item>
    <List.Item
      title="Turn bluetooth off"
      icon={Icon.CircleDisabled}
      actions={
        <ActionPanel>
          <Action
            title="Turn bluetooth off"
            onAction={() => {
              openDeepLink(
                "vicinae://launch/@Gelei/store.vicinae.bluetooth/bluetooth-off",
              );
            }}
          />
        </ActionPanel>
      }
    ></List.Item>
  </List>
);
