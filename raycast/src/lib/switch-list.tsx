import {
  Action,
  ActionPanel,
  Color,
  Icon,
  Keyboard,
  List,
  showToast,
  Toast,
} from "@raycast/api";
import { usePromise } from "@raycast/utils";
import { Device, listDevices, switchDevice } from "./audio";

interface SwitchListProps {
  /** Which side to switch: input devices or output devices. */
  kind: "Input" | "Output";
}

export function SwitchList({ kind }: SwitchListProps) {
  const { data, isLoading, revalidate } = usePromise(listDevices);

  const isInput = kind === "Input";
  const devices = (data?.devices ?? []).filter((d) => d.type === kind);
  const currentName = isInput ? data?.default_input : data?.default_output;

  async function onSwitch(device: Device) {
    const toast = await showToast({
      style: Toast.Style.Animated,
      title: `Switching ${kind.toLowerCase()} to ${device.name}…`,
    });
    try {
      const result = await switchDevice(device.name, isInput);
      if (result.success) {
        toast.style = Toast.Style.Success;
        toast.title = `${kind} set to ${device.name}`;
      } else {
        toast.style = Toast.Style.Failure;
        toast.title = "Switch failed";
        toast.message = result.error ?? "Unknown error";
      }
    } catch (err) {
      toast.style = Toast.Style.Failure;
      toast.title = "Switch failed";
      toast.message = err instanceof Error ? err.message : String(err);
    } finally {
      await revalidate();
    }
  }

  return (
    <List
      isLoading={isLoading}
      searchBarPlaceholder={`Filter ${kind.toLowerCase()} devices…`}
    >
      {devices.map((device) => {
        const isCurrent = device.name === currentName;
        return (
          <List.Item
            key={`${device.type}:${device.id}:${device.uid ?? device.name}`}
            icon={isInput ? Icon.Microphone : Icon.Speaker}
            title={device.name}
            accessories={
              isCurrent
                ? [
                    {
                      icon: {
                        source: Icon.CheckCircle,
                        tintColor: Color.Green,
                      },
                      text: "Current",
                    },
                  ]
                : []
            }
            actions={
              <ActionPanel>
                <Action
                  title={`Set as Default ${kind}`}
                  icon={Icon.Checkmark}
                  onAction={() => onSwitch(device)}
                />
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
      })}
      <List.EmptyView title="No devices found" icon={Icon.SpeakerOff} />
    </List>
  );
}
