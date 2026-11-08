// This work is all offloaded to the PulseAudio extension (https://www.vicinae.com/extensions/rastsislaux/pulseaudio)
// so this menu will just route you to that extension. It must be
// installed for this to work properly.

import { Action, ActionPanel, Icon, List } from "@vicinae/api";
import { JSX } from "react/jsx-runtime";
import { openDeepLink } from "./util";

export const AudioMenu: JSX.Element = (
  <List searchBarPlaceholder={"Search menu..."}>
    <List.Section title={"Quick Access"}></List.Section>
    <List.Item
      title="Outputs"
      icon={Icon.Speaker}
      actions={
        <ActionPanel>
          <Action
            title="Outputs"
            onAction={() => {
              openDeepLink(
                "vicinae://launch/@rastsislaux/store.vicinae.pulseaudio/outputDevices",
              );
            }}
          />
        </ActionPanel>
      }
    ></List.Item>
    <List.Item
      title="Inputs"
      icon={Icon.Microphone}
      actions={
        <ActionPanel>
          <Action
            title="Inputs"
            onAction={() => {
              openDeepLink(
                "vicinae://launch/@rastsislaux/store.vicinae.pulseaudio/inputDevices",
              );
            }}
          />
        </ActionPanel>
      }
    ></List.Item>
    <List.Item
      title="Playback"
      icon={Icon.PlayFilled}
      actions={
        <ActionPanel>
          <Action
            title="Playback"
            onAction={() => {
              openDeepLink(
                "vicinae://launch/@rastsislaux/store.vicinae.pulseaudio/playback",
              );
            }}
          />
        </ActionPanel>
      }
    ></List.Item>
    <List.Item
      title="Recording"
      icon={Icon.CircleFilled}
      actions={
        <ActionPanel>
          <Action
            title="Recording"
            onAction={() => {
              openDeepLink(
                "vicinae://launch/@rastsislaux/store.vicinae.pulseaudio/recording",
              );
            }}
          />
        </ActionPanel>
      }
    ></List.Item>
    <List.Item
      title="Configure devices"
      icon={Icon.Cog}
      actions={
        <ActionPanel>
          <Action
            title="Configure devices"
            onAction={() => {
              openDeepLink(
                "vicinae://launch/@rastsislaux/store.vicinae.pulseaudio/configuration",
              );
            }}
          />
        </ActionPanel>
      }
    ></List.Item>
  </List>
);
