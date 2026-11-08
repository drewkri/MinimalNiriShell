import { Action, ActionPanel, closeMainWindow, Icon, List } from "@vicinae/api";
import { JSX } from "react/jsx-runtime";
import { execAsync } from "./util";

export const PowerMenu: JSX.Element = (
  <List searchBarPlaceholder={"Search menu..."}>
    <List.Section title={"Quick Access"}></List.Section>
    <List.Item
      title="Shutdown"
      icon={Icon.Power}
      actions={
        <ActionPanel>
          <Action
            title="Shut down computer"
            onAction={() => {
              execAsync("systemctl poweroff");
              closeMainWindow();
            }}
          />
        </ActionPanel>
      }
    ></List.Item>
    <List.Item
      title="Reboot"
      icon={Icon.Redo}
      actions={
        <ActionPanel>
          <Action
            title="Reboot computer"
            onAction={() => {
              execAsync("systemctl reboot");
              closeMainWindow();
            }}
          />
        </ActionPanel>
      }
    ></List.Item>
    <List.Item
      title="Log out"
      icon={Icon.Logout}
      actions={
        <ActionPanel>
          <Action
            title="Log this user out"
            onAction={() => {
              execAsync("loginctl terminate-user $USER");
              closeMainWindow();
            }}
          />
        </ActionPanel>
      }
    ></List.Item>
    <List.Item
      title="Lock"
      icon={Icon.Lock}
      actions={
        <ActionPanel>
          <Action
            title="Lock this computer"
            onAction={() => {
              execAsync("loginctl lock-session");
              closeMainWindow();
            }}
          />
        </ActionPanel>
      }
    ></List.Item>
    <List.Item
      title="Soft reboot"
      icon={Icon.Redo}
      actions={
        <ActionPanel>
          <Action
            title="Soft reboot computer"
            onAction={() => {
              execAsync("systemctl soft-reboot");
              closeMainWindow();
            }}
          />
        </ActionPanel>
      }
    ></List.Item>
    <List.Item
      title="Sleep"
      icon={Icon.Moon}
      actions={
        <ActionPanel>
          <Action
            title="Put computer in sleep mode"
            onAction={() => {
              execAsync("systemctl sleep");
              closeMainWindow();
            }}
          />
        </ActionPanel>
      }
    ></List.Item>
    <List.Item
      title="Suspend"
      icon={Icon.Pause}
      actions={
        <ActionPanel>
          <Action
            title="Suspend computer"
            onAction={() => {
              execAsync("systemctl suspend");
              closeMainWindow();
            }}
          />
        </ActionPanel>
      }
    ></List.Item>
    <List.Item
      title="Hibernate"
      icon={Icon.Stop}
      actions={
        <ActionPanel>
          <Action
            title="Hibernate computer"
            onAction={() => {
              execAsync("systemctl hibernate");
              closeMainWindow();
            }}
          />
        </ActionPanel>
      }
    ></List.Item>
  </List>
);
