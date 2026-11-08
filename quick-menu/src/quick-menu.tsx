import {
  Action,
  ActionPanel,
  closeMainWindow,
  Color,
  Detail,
  Icon,
  KeyEquivalent,
  List,
  showToast,
  Toast,
} from "@vicinae/api";
import { getCalendar } from "./calendar";
import {
  executePlaybackAction,
  getPlayerMetadata,
  PlayerMetadata,
} from "./media-player";

import { currentBattery, currentTime } from "./sysinfo";
import React, { useState, useEffect, JSX } from "react";
import { PowerMenu } from "./power-menu";
import { CurrentPreferences, execAsync } from "./util";
import { AudioMenu } from "./audio-menu";
import { NetworkMenu } from "./wifi-menu";
import { BluetoothMenu } from "./bluetooth-menu";

// React component for the media player
const buildMediaPlayerRow = (): JSX.Element | null => {
  const [metadata, setMetadata] = useState<PlayerMetadata | null>(null);

  const updateMetadata = (): void => {
    getPlayerMetadata().then((maybeData) => {
      setMetadata(maybeData);
    });
  };
  useEffect(() => {
    updateMetadata();

    // Updates every second
    const intervalId = setInterval(() => {
      updateMetadata();
    }, 1000);

    return () => clearInterval(intervalId);
  }, []);

  if (metadata) {
    return (
      <List.Item
        title={metadata.title ?? "--unknown title--"}
        icon={metadata.albumArt ?? Icon.Music}
        accessories={[
          {
            text: `${metadata.position ?? "--"}/${metadata.songLength ?? "--"}`,
          },
        ]}
        actions={
          <ActionPanel>
            <Action
              title={(metadata.paused ?? true) ? "Play" : "Pause"}
              onAction={() => {
                executePlaybackAction(
                  (metadata.paused ?? true) ? "play" : "pause",
                );
                updateMetadata();
              }}
            />
            <Action
              title="Next"
              icon={Icon.ForwardFilled}
              onAction={() => {
                executePlaybackAction("next");
                updateMetadata();
              }}
            />
            <Action
              title="Previous"
              icon={Icon.RewindFilled}
              onAction={() => {
                executePlaybackAction("previous");
                updateMetadata();
              }}
            />
          </ActionPanel>
        }
        detail={
          <List.Item.Detail
            metadata={
              <List.Item.Detail.Metadata>
                <List.Item.Detail.Metadata.Label
                  title="Status"
                  text={`${(metadata.paused ?? true) ? "Paused" : "Playing"}`}
                />
                <List.Item.Detail.Metadata.Label
                  title="Title"
                  text={`${metadata.title ?? "--unknown--"}`}
                />
                <List.Item.Detail.Metadata.Label
                  title="Album"
                  text={`${metadata.album ?? "--unknown--"}`}
                />
                <List.Item.Detail.Metadata.Label
                  title="Artist"
                  text={`${metadata.artist ?? "--unknown--"}`}
                />
                <List.Item.Detail.Metadata.Label
                  title="Duration"
                  text={`${metadata.songLength ?? "--"}`}
                />
                <List.Item.Detail.Metadata.Label
                  title="Art URL"
                  text={`${metadata.albumArt ?? "None Found"}`}
                />
              </List.Item.Detail.Metadata>
            }
          />
        }
      />
    );
  } else {
    return null;
  }
};

export default function QuickMenu() {
  const listItems = options.map((option) => (
    <List.Item
      key={option.name}
      title={option.name}
      icon={option.icon}
      detail={
        <List.Item.Detail
          markdown={getCalendar()}
          metadata={
            <List.Item.Detail.Metadata>
              <Detail.Metadata.Label
                title="Current Time"
                text={currentTime()}
              />
              {CurrentPreferences.showBattery ? (
                <Detail.Metadata.Label
                  title="Battery"
                  text={currentBattery()}
                />
              ) : null}
            </List.Item.Detail.Metadata>
          }
        />
      }
      actions={<ActionPanel>{option.action}</ActionPanel>}
    />
  ));
  // Media player
  const mediaRow = buildMediaPlayerRow();
  if (mediaRow) {
    listItems.push(mediaRow);
  }

  return (
    <List isShowingDetail searchBarPlaceholder={"Search menu..."}>
      {listItems}
      <List.Section title={"Quick Access"}></List.Section>
    </List>
  );
}

type Option = {
  name: string;
  icon: string;
  action: JSX.Element;
};

export const options: Option[] = [
  {
    name: "Notifications",
    icon: Icon.Bell,
    action: (
      <Action
        title="View notifications"
        onAction={() => {
          execAsync(CurrentPreferences.notificationsCommand);
          closeMainWindow();
        }}
      ></Action>
    ),
  },
  {
    name: "Audio Settings",
    icon: Icon.Speaker,
    action: (
      <Action.Push title="Open audio settings" target={AudioMenu}></Action.Push>
    ),
  },
  {
    name: "Network Settings",
    icon: Icon.Network,
    action: (
      <Action.Push
        title="Open network settings"
        target={NetworkMenu}
      ></Action.Push>
    ),
  },
  {
    name: "Bluetooth Settings",
    icon: Icon.Bluetooth,
    action: (
      <Action.Push
        title="Open bluetooth settings"
        target={BluetoothMenu}
      ></Action.Push>
    ),
  },
  {
    name: "System Tray",
    icon: Icon.Tray,
    action: (
      <Action
        title="View System Tray"
        onAction={() => {
          execAsync("vicinae vicinae://launch/core/search-tray");
        }}
      ></Action>
    ),
  },
  {
    name: "Power Menu",
    icon: Icon.Power,
    action: (
      <Action.Push title="Open power menu" target={PowerMenu}></Action.Push>
    ),
  },
];
