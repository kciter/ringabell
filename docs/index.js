import init, { register, search } from "./ringabell.js";
import { FFmpeg } from "./ffmpeg/ffmpeg/index.js";
import { toBlobURL } from "./ffmpeg/util/index.js";

async function loadFFmpeg() {
  const baseURL = "https://unpkg.com/@ffmpeg/core@0.12.6/dist/umd";
  const ffmpeg = new FFmpeg();

  await ffmpeg.load({
    coreURL: await toBlobURL("/ffmpeg/core/ffmpeg-core.js", "text/javascript"),
    wasmURL: await toBlobURL(
      "/ffmpeg/core/ffmpeg-core.wasm",
      "application/wasm"
    ),
  });

  return ffmpeg;
}

async function main() {
  await init(); // Wasm 모듈 초기화

  const ffmpeg = await loadFFmpeg();

  const registerButton = document.getElementById("registerButton");
  const searchButton = document.getElementById("searchButton");
  const recordButton = document.getElementById("recordButton");
  const output = document.getElementById("output");
  let registeredSongNames = [];

  registerButton.addEventListener("click", () => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = "audio/wav";
    input.click();

    input.addEventListener("change", async (event) => {
      const file = event.target.files[0];
      if (file && file.type === "audio/wav") {
        output.textContent = "Registering song...";

        const reader = new FileReader();
        reader.onload = (e) => {
          const songData = new Uint8Array(e.target.result);
          const songName = file.name;
          register(songName, songData);

          registeredSongNames.push(songName);
          updateRegisteredFileList();

          output.textContent = `Registered song: ${songName}`;
        };
        reader.readAsArrayBuffer(file);
      } else {
        output.textContent = "Please select a valid WAV file.";
      }
    });
  });

  async function initialRegisteredFiles() {
    const registeredFiles = document.getElementById("registeredFiles");
    registeredFiles.innerHTML = ""; // 기존 목록 초기화

    const songNames = [
      "Burlesque - National Sweetheart.wav",
      "Corny Candy - The Soundlings.wav",
      "Head of The Snake - Everet Almond.wav",
      "July - John Patitucci.wav",
      "Walking The Dog - Jeremy Korpas.wav",
    ];

    for (const songName of songNames) {
      const response = await fetch(`./public/${songName}`);
      const arrayBuffer = await response.arrayBuffer();
      const songData = new Uint8Array(arrayBuffer);
      await register(songName, songData);
      registeredSongNames.push(songName);
      updateRegisteredFileList();
    }
  }

  await initialRegisteredFiles();

  function updateRegisteredFileList() {
    const registeredFiles = document.getElementById("registeredFiles");
    registeredFiles.innerHTML = "";
    registeredSongNames.forEach((name) => {
      const li = document.createElement("li");
      li.textContent = name;
      registeredFiles.appendChild(li);
    });
  }

  searchButton.addEventListener("click", () => {
    const fileInput = document.createElement("input");
    fileInput.type = "file";
    fileInput.accept = "audio/wav";
    fileInput.click();

    fileInput.addEventListener("change", async (event) => {
      const file = event.target.files[0];
      if (file && file.type === "audio/wav") {
        output.textContent = "Searching song...";

        const reader = new FileReader();
        reader.onload = async (e) => {
          const songData = new Uint8Array(e.target.result);
          const result = JSON.parse(await search(songData));

          console.log(result);

          if (result.score === 0) {
            output.textContent = "Not found";
          } else {
            output.textContent = `Search result: ${result.songName}`;
          }
        };

        reader.readAsArrayBuffer(file);
      } else {
        output.textContent = "Please select a valid WAV file.";
      }
    });
  });

  recordButton.addEventListener("click", async () => {
    output.textContent = "Listening...";

    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      const audioContext = new AudioContext();

      if (!window.MediaRecorder) {
        output.textContent = "MediaRecorder not supported in this browser.";
        return;
      }

      const mediaRecorder = new MediaRecorder(stream);
      const audioChunks = [];

      mediaRecorder.ondataavailable = (event) => {
        audioChunks.push(event.data);
      };

      mediaRecorder.onstop = async () => {
        const audioBlob = new Blob(audioChunks, { type: "audio/webm" });
        const audioBytes = new Uint8Array(await audioBlob.arrayBuffer());

        // const wavData = await convertWebmToWav(arrayBuffer, audioContext);

        await ffmpeg.writeFile("input.webm", audioBytes);
        await ffmpeg.exec([
          "-i",
          "input.webm",
          "-acodec",
          "pcm_s16le",
          "-ac",
          "1",
          "-ar",
          "44100",
          "output.wav",
        ]);
        const data = await ffmpeg.readFile("output.wav");

        const result = JSON.parse(await search(data));
        // // download wavData
        // const blob = new Blob([wavData], { type: "audio/wav" });
        // const url = URL.createObjectURL(blob);
        // const a = document.createElement("a");
        // a.href = url;
        // a.download = "recorded.wav";
        // a.click();

        console.log(result);

        if (result.score === 0) {
          output.textContent = "Not found";
        } else {
          output.textContent = `Search result: ${result.songName}`;
        }
      };

      mediaRecorder.start();
      setTimeout(() => {
        mediaRecorder.stop();
        output.textContent = "Processing...";
      }, 10000);
    } catch (error) {
      output.textContent = "Error accessing microphone: " + error.message;
    }
  });
}

main();
