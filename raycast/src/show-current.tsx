import { Action, ActionPanel, Color, Icon, Keyboard, List } from "@raycast/api";
import { usePromise } from "@raycast/utils";
import { CurrentDevice, showCurrent } from "./lib/audio";

export default function Command() {
  const { data, isLoading, revalidate } = usePromise(showCurrent);

  const row = (
    label: string,
    icon: Icon,
    device: CurrentDevice | null | undefined,
  ) => (
    <List.Item
      icon={icon}
      title={label}
      subtitle={device?.name ?? "None"}
      accessories={
        device
          ? [
              {
                icon: { source: Icon.Dot, tintColor: Color.Green },
                text: device.type,
              },
            ]
          : [{ text: "Unavailable" }]
      }
      actions={
        <ActionPanel>
          {device ? (
            <Action.CopyToClipboard
              title="Copy Device Name"
              content={device.name}
            />
          ) : null}
          <Action
            title="Refresh"
            icon={Icon.ArrowClockwise}
            shortcut={Keyboard.Shortcut.Common.Refresh}
            onAction={() => revalidate()}
          />
        </ActionPanel>
      }
    />
  );

  return (
    <List isLoading={isLoading}>
      {row("Output", Icon.Speaker, data?.output)}
      {row("Input", Icon.Microphone, data?.input)}
    </List>
  );
}
