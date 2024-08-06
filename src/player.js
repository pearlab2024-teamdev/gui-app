let activeNotes = new Set();
let eventQueue = new Queue(512);
// const variables
const CANVAS_WIDTH = 900 / 2;
const CANVAS_HEIGHT = 400 / 2;
const TIME_DISPLAY_HEIGHT = 20; // 経過時間表示のための高さ
const PIANO_HEIGHT = (CANVAS_HEIGHT - TIME_DISPLAY_HEIGHT) / 2; // 鍵盤のための高さを2段に分ける
const WHITE_KEY_WIDTH = CANVAS_WIDTH / 21; // 3オクターブ分の白鍵を横に収める
const WHITE_KEY_HEIGHT = PIANO_HEIGHT; // 白鍵の高さを全体に合わせて調整
const BLACK_KEY_WIDTH = WHITE_KEY_WIDTH * 0.6;
const BLACK_KEY_HEIGHT = WHITE_KEY_HEIGHT * 0.6;
// ピアノロールの描画(switchPlayer関数内で呼び出し)
function drawPianoRoll() {
  const canvas = document.getElementById('pianoRoll');
  canvas.height = CANVAS_HEIGHT; // Canvasの高さを設定
  canvas.width = CANVAS_WIDTH; // Canvasの幅を設定

  const ctx = canvas.getContext('2d');
  
  const numOctaves = 6; // 6オクターブ表示に調整

  const whiteKeys = [0, 2, 4, 5, 7, 9, 11];
  const blackKeys = [1, 3, 6, 8, 10];
  const module_3_table = [0,1,2,0];
  let startTime = null;

  // ピアノの描画
  function drawPiano() {
    // 先に全ての白鍵を描画
    for (let octave = 0; octave < numOctaves; octave++) {
      const octaveOffsetX = module_3_table[(octave % 3)] * 7 * WHITE_KEY_WIDTH;
      const yOffset = Math.floor(octave / 3) * PIANO_HEIGHT;
      // Note: 破壊的な変更(復帰可能にするためにコメントで保護)
      for(let i = 0,_end = whiteKeys.length; i < _end; i++) {
        let key = whiteKeys[i];
        const pitch = octave * 12 + key + 24; // C1から開始するように24を追加
        const x = octaveOffsetX + i * WHITE_KEY_WIDTH;
        const y = yOffset;
        ctx.fillStyle = activeNotes.has(pitch) ? '#D3D3D3' : 'white';  // 薄い灰色でアクティブな白鍵を表示
        ctx.fillRect(x, y, WHITE_KEY_WIDTH, WHITE_KEY_HEIGHT);
        ctx.strokeStyle = 'black';
        ctx.strokeRect(x, y, WHITE_KEY_WIDTH, WHITE_KEY_HEIGHT);

        // Cのラベルを表示
        if (key === 0) {
          ctx.fillStyle = 'black';
          ctx.font = '14px Arial';
          ctx.fillText(`C${Math.floor(pitch / 12) - 1}`, x + 5, y + WHITE_KEY_HEIGHT - 5);
        }
      }
      // whiteKeys.forEach((key, i) => {
      //   const pitch = octave * 12 + key + 24; // C1から開始するように24を追加
      //   const x = octaveOffsetX + i * WHITE_KEY_WIDTH;
      //   const y = yOffset;
      //   ctx.fillStyle = activeNotes.has(pitch) ? '#D3D3D3' : 'white';  // 薄い灰色でアクティブな白鍵を表示
      //   ctx.fillRect(x, y, WHITE_KEY_WIDTH, WHITE_KEY_HEIGHT);
      //   ctx.strokeStyle = 'black';
      //   ctx.strokeRect(x, y, WHITE_KEY_WIDTH, WHITE_KEY_HEIGHT);

      //   // Cのラベルを表示
      //   if (key === 0) {
      //     ctx.fillStyle = 'black';
      //     ctx.font = '14px Arial';
      //     ctx.fillText(`C${Math.floor(pitch / 12) - 1}`, x + 5, y + WHITE_KEY_HEIGHT - 5);
      //   }
      // });
    }

    // 黒鍵を描画し、その上にアクティブな色を適用
    for (let octave = 0; octave < numOctaves; octave++) {
      const octaveOffsetX = module_3_table[(octave % 3)] * 7 * WHITE_KEY_WIDTH;
      const yOffset = Math.floor(octave / 3) * PIANO_HEIGHT;
      // Note: 破壊的な変更(復帰可能にするためにコメントで保護)
      for(let i = 0,_end = blackKeys.length; i < _end; i++) {
        let key = blackKeys[i];
        const pitch = octave * 12 + key + 24; // C#1から開始するように24を追加
        const x = octaveOffsetX + whiteKeys.indexOf(key - 1) * WHITE_KEY_WIDTH + WHITE_KEY_WIDTH - BLACK_KEY_WIDTH / 2;
        const y = yOffset;
        // 黒鍵の基本描画
        ctx.fillStyle = activeNotes.has(pitch) ? '#A9A9A9' : 'black';  // 濃い灰色でアクティブな黒鍵を表示
        ctx.fillRect(x, y, BLACK_KEY_WIDTH, BLACK_KEY_HEIGHT);
        ctx.strokeStyle = 'black';
        ctx.strokeRect(x, y, BLACK_KEY_WIDTH, BLACK_KEY_HEIGHT);
      }
      // blackKeys.forEach((key) => {
      //   const pitch = octave * 12 + key + 24; // C#1から開始するように24を追加
      //   const x = octaveOffsetX + whiteKeys.indexOf(key - 1) * WHITE_KEY_WIDTH + WHITE_KEY_WIDTH - BLACK_KEY_WIDTH / 2;
      //   const y = yOffset;

      //   // 黒鍵の基本描画
      //   ctx.fillStyle = activeNotes.has(pitch) ? '#A9A9A9' : 'black';  // 濃い灰色でアクティブな黒鍵を表示
      //   ctx.fillRect(x, y, BLACK_KEY_WIDTH, BLACK_KEY_HEIGHT);
      //   ctx.strokeStyle = 'black';
      //   ctx.strokeRect(x, y, BLACK_KEY_WIDTH, BLACK_KEY_HEIGHT);
      // });
    }
  }

  // 経過時間の表示
  function displayTime(elapsedTime) {
    ctx.fillStyle = 'black';
    ctx.font = '16px Arial';
    // 秒数は小数点以下1桁まで表示
    ctx.fillText(`Time: ${Math.floor(elapsedTime / 1000)}s`, 10, canvas.height - 5);
  }

  // アニメーションの開始
  function animate(time) {
    if (!startTime) startTime = time;
    const elapsedTime = time - startTime;

    ctx.clearRect(0, 0, canvas.width, canvas.height);
    drawPiano();
    displayTime(elapsedTime);

    processEventQueue();

    requestAnimationFrame(animate);
  }

  requestAnimationFrame(animate);

  // ノートオンイベントを処理
  function noteOn(pitch) {
    if (activeNotes.has(pitch)) return;
    activeNotes.add(pitch);
  }

  // ノートオフイベントを処理
  function noteOff(pitch) {
    activeNotes.delete(pitch);
    if (activeNotes.has(pitch)) {
      noteOff(pitch);
    }
  }

  // 外部からノートオン、ノートオフをトリガーできるようにする
  window.noteOn = noteOn;
  window.noteOff = noteOff;
}

// イベントキューの処理
function processEventQueue() {
  while (eventQueue.length > 0) {
    const data = eventQueue.dequeue();
    handleEvent(data);
  }
}

// イベントデータをキューに追加
function updatePianoRoll(data) {
  eventQueue.enqueue(data);
  const playerConsoleArea = document.getElementById('playerConsole');
  if (playerConsoleArea) {
    playerConsoleArea.value = `Playback Data: key: ${data.key}, vel: ${data.vel}\n`;
    playerConsoleArea.scrollTop = playerConsoleArea.scrollHeight;
  }
}

// イベントデータの処理
function handleEvent(data) {
  const playerConsoleArea = document.getElementById('playerConsole');
  // playerConsoleArea.value += `Playback Data: key: ${data.key}, vel: ${data.vel}\n`;
  playerConsoleArea.scrollTop = playerConsoleArea.scrollHeight;
  if(data.vel !== 0) {
    // console.log(`DATA | vel: ${data.vel}, key: ${data.key}\n`)
    window.noteOn(data.key);
  } else if(data.vel == 0) {
    // console.log(`DATA | vel: ${data.vel}, key: ${data.key}\n`)
    window.noteOff(data.key);
  } else {
    // その他のイベントを処理
    console.warn("Unknown event type:", data);
  }
}

function updateTempo(data) {
  // テンポ情報をパースして更新
  // const tempoMatch = data.match(/tempo: (\d+)/);
  if (data.tempo) {
    // const tempo = parseInt(tempoMatch[1], 10);
    const tempoDisplay = document.getElementById('tempoDisplay');
    if (tempoDisplay) {
      tempoDisplay.textContent = `Tempo: ${data.tempo}`;
    }
  }
}

function handleEndEvent() {
  activeNotes.clear();
  console.log("End Event received.");
}

