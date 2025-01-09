import { createSignal} from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [inputAudioDevices, setInputAudioDevices] = createSignal<string[]>([]);
  const [outputAudioDevices, setOutputAudioDevices] = createSignal<string[]>([]);
  const [selectedInputDevice, setSelectedInputDevice] = createSignal<string | null>(null);
  const [selectedOutputDevice, setSelectedOutputDevice] = createSignal<string | null>(null);

  // 入力オーディオデバイスを取得する関数
  const fetchInputAudioDevices = async () => {
    try {
      const devices = await invoke<string[]>("get_input_audio_devices");
      setInputAudioDevices(devices);
    } catch (error) {
      console.error("Error fetching input audio devices:", error);
    }
  };

  // 出力オーディオデバイスを取得する関数
  const fetchOutputAudioDevices = async () => {
    try {
      const devices = await invoke<string[]>("get_output_audio_devices");
      setOutputAudioDevices(devices);
    } catch (error) {
      console.error("Error fetching output audio devices:", error);
    }
  };

  // デバイス選択時の処理
  const selectInputDevice = async (deviceName: string) => {
    try {
      await invoke("set_input_device", { inputDeviceName: deviceName }).then(()=>{
        setSelectedInputDevice(deviceName); // 選択中のデバイスを更新
        console.log(`Input device set to: ${deviceName}`);
      });
    } catch (error) {
      console.error(`Failed to set input device (${deviceName}):`, error);
    }
  };
  const selectOutputDevice = async (deviceName: string) => {
    try {
      await invoke("set_output_device", { outputDeviceName: deviceName }).then(()=>{
        setSelectedOutputDevice(deviceName); // 選択中のデバイスを更新
        console.log(`Output device set to: ${deviceName}`);
      });
    } catch (error) {
      console.error(`Failed to set output device (${deviceName}):`, error);
    }
  };

  return (
    <main class="container">
      <h1>TauriAudio</h1>
      <div>
        <button onClick={fetchInputAudioDevices}>入力オーディオデバイス取得</button>
        <ul>
          {inputAudioDevices().length > 0 ? (
            inputAudioDevices().map((device) => (
              <li onClick={()=>selectInputDevice(device)} style={{color: selectedInputDevice()===device?"#F00":"#000"}}>
                {device}
              </li>))
          ) : (
            <li>入力デバイスが見つかりませんでした</li>
          )}
        </ul>
      </div>
      <div>
        <button onClick={fetchOutputAudioDevices}>出力オーディオデバイス取得</button>
        <ul>
          {outputAudioDevices().length > 0 ? (
            outputAudioDevices().map((device) => (
              <li onClick={()=>selectOutputDevice(device)} style={{color: selectedOutputDevice()===device?"#F00":"#000"}}>
                {device}
              </li>))
          ) : (
            <li>出力デバイスが見つかりませんでした</li>
          )}
        </ul>
      </div>
    </main>
  );
}

export default App;
