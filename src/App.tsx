import { Channel, invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import "./App.css";
import emote from './assets/crying-yt.png';

type AudioFeature = { rms: number; zcr: number };

function App() {
  const [feature, setFeature] = useState<AudioFeature>({ rms: 0, zcr: 0 });
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const channel = new Channel<AudioFeature>();
    channel.onmessage = (msg) => setFeature(msg);

    invoke("start_audio_stream", { onFeature: channel }).catch((e) =>
      setError(String(e)),
    );

    return () => {
      invoke("stop_audio_stream").catch(console.error);
    };
  }, []);

  return (
    <main className="container">
      <img
        className="emote"
        src={emote}
        alt="emote"
        style={{
          width: `${Math.min(100, feature.rms * 800)}vw`,
          height: `${Math.min(100, feature.zcr * 1500)}vh`,
        }}
      />

      <div className="hud">
        {error && <p style={{ color: "tomato" }}>{error}</p>}
        <pre>RMS: {feature.rms.toFixed(4)}</pre>
        <pre>ZCR: {feature.zcr.toFixed(4)}</pre>
      </div>
    </main>
  );
}

export default App;
