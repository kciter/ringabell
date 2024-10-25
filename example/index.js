import init, { register, search } from "./ringabell.js";

async function main() {
  await init(); // Wasm 모듈 초기화

  const fileInput = document.getElementById("fileInput");
  const registerButton = document.getElementById("registerButton");
  const searchButton = document.getElementById("searchButton");
  const recordButton = document.getElementById("recordButton");
  const output = document.getElementById("output");
  let registeredSongNames = [];

  registerButton.addEventListener("click", () => {
    fileInput.click(); // 파일 선택 대화상자 열기
  });

  // 파일 선택 후 이벤트 처리
  fileInput.addEventListener("change", async (event) => {
    const file = event.target.files[0];
    if (file && file.type === "audio/wav") {
      // WAV 파일인지 확인
      output.textContent = "Registering song...";

      // 파일을 읽고 Wasm에 등록
      const reader = new FileReader();
      reader.onload = (e) => {
        const songData = new Uint8Array(e.target.result);
        const songName = file.name;

        // Wasm의 register 함수를 호출하여 파일 등록
        register(songName, songData);

        // 등록된 파일 목록에 추가
        registeredSongNames.push(songName);
        updateRegisteredFileList();

        output.textContent = `Registered song: ${songName}`;
      };
      reader.readAsArrayBuffer(file);
    } else {
      output.textContent = "Please select a valid WAV file.";
    }
  });

  function updateRegisteredFileList() {
    registeredFiles.innerHTML = ""; // 기존 목록 초기화
    registeredSongNames.forEach((name) => {
      const li = document.createElement("li");
      li.textContent = name;
      registeredFiles.appendChild(li);
    });
  }

  // 마이크로 녹음하여 WAV 데이터로 변환 후 탐색
  recordButton.addEventListener("click", async () => {
    output.textContent = "Listening...";

    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      const audioContext = new AudioContext();
      const mediaRecorder = new MediaRecorder(stream);
      const audioChunks = [];

      mediaRecorder.ondataavailable = (event) => {
        audioChunks.push(event.data);
      };

      mediaRecorder.onstop = async () => {
        const audioBlob = new Blob(audioChunks);
        const arrayBuffer = await audioBlob.arrayBuffer();

        // PCM 데이터를 WAV 파일로 변환
        const wavData = convertToWav(
          new Float32Array(await audioContext.decodeAudioData(arrayBuffer))
        );

        // `search` 함수를 호출하여 WAV 데이터를 전달
        const result = JSON.parse(search(wavData));

        console.log(result);

        // 스코어 기반 검색 결과 처리
        if (result.score === 0) {
          output.textContent = "Not found";
        } else {
          output.textContent = `Search result: ${result.songName}`;
        }
      };

      // 5초간 녹음 후 자동 중지
      mediaRecorder.start();
      setTimeout(() => {
        mediaRecorder.stop();
        output.textContent = "Processing...";
      }, 10000);
    } catch (error) {
      output.textContent = "Error accessing microphone: " + error.message;
    }
  });

  // PCM 데이터를 WAV 파일 포맷으로 변환
  function convertToWav(pcmData) {
    const sampleRate = 44100;
    const numChannels = 1;
    const bitsPerSample = 16;
    const byteRate = (sampleRate * numChannels * bitsPerSample) / 8;
    const blockAlign = (numChannels * bitsPerSample) / 8;
    const wavBuffer = new ArrayBuffer(44 + pcmData.length * 2); // 44 bytes header + PCM data
    const view = new DataView(wavBuffer);

    // WAV 파일 헤더 작성
    writeString(view, 0, "RIFF");
    view.setUint32(4, 36 + pcmData.length * 2, true);
    writeString(view, 8, "WAVE");
    writeString(view, 12, "fmt ");
    view.setUint32(16, 16, true);
    view.setUint16(20, 1, true); // PCM 형식
    view.setUint16(22, numChannels, true);
    view.setUint32(24, sampleRate, true);
    view.setUint32(28, byteRate, true);
    view.setUint16(32, blockAlign, true);
    view.setUint16(34, bitsPerSample, true);
    writeString(view, 36, "data");
    view.setUint32(40, pcmData.length * 2, true);

    // PCM 데이터를 WAV 포맷에 맞게 작성
    let offset = 44;
    for (let i = 0; i < pcmData.length; i++) {
      const sample = Math.max(-1, Math.min(1, pcmData[i]));
      view.setInt16(
        offset,
        sample < 0 ? sample * 0x8000 : sample * 0x7fff,
        true
      );
      offset += 2;
    }

    return new Uint8Array(wavBuffer);
  }

  function writeString(view, offset, string) {
    for (let i = 0; i < string.length; i++) {
      view.setUint8(offset + i, string.charCodeAt(i));
    }
  }
}

main();
