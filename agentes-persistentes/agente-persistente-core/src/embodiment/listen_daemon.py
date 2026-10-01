#!/usr/bin/env python3
"""
listen_daemon.py — Escuta Contínua com Voice Activity Detection (VAD) & Antiruído

Usa webrtcvad no nível 3 (máxima agressividade antirruído) e cálculo de RMS para filtrar
ruídos de fundo, estática e respiração, gravando apenas fala humana legítima.
"""

import sys
import wave
import collections
import subprocess
import struct
import math
import webrtcvad
import speech_recognition as sr

SAMPLE_RATE = 16000       # Taxa 16kHz
FRAME_DURATION_MS = 30    # 30ms por frame
FRAME_SIZE_BYTES = int(SAMPLE_RATE * 2 * FRAME_DURATION_MS / 1000)  # 960 bytes (16-bit mono)
VAD_AGGRESSIVENESS = 3    # Nível 3: Máximo filtro antirruído (descarta estática/ar condicionado)
SILENCE_THRESHOLD_FRAMES = int(1200 / FRAME_DURATION_MS)  # ~1.2s de silêncio
SPEECH_MIN_FRAMES = int(360 / FRAME_DURATION_MS)          # Mínimo 360ms de fala humana
AUDIO_OUTPUT_PATH = "/tmp/mic_input.wav"


def calc_rms(data):
    if len(data) < 2:
        return 0
    count = len(data) // 2
    samples = struct.unpack(f"{count}h", data)
    sum_squares = sum(s * s for s in samples)
    return math.sqrt(sum_squares / count)


def record_until_silence():
    vad = webrtcvad.Vad(VAD_AGGRESSIVENESS)

    pre_buffer_size = int(300 / FRAME_DURATION_MS)
    ring_buffer = collections.deque(maxlen=pre_buffer_size)

    speech_frames = []
    is_speaking = False
    silence_count = 0
    speech_count = 0

    print("🎧 [ESCUTA CONTÍNUA] Aguardando sua voz...", file=sys.stderr, flush=True)

    proc = subprocess.Popen(
        [
            "ffmpeg",
            "-loglevel", "quiet",
            "-f", "pulse",
            "-i", "default",
            "-ar", str(SAMPLE_RATE),
            "-ac", "1",
            "-f", "s16le",
            "-"
        ],
        stdout=subprocess.PIPE
    )

    try:
        while True:
            frame_bytes = proc.stdout.read(FRAME_SIZE_BYTES)
            if len(frame_bytes) < FRAME_SIZE_BYTES:
                break

            # Filtro Antirruído duplo: VAD Nível 3 + Amplitude RMS mínima
            try:
                is_speech = vad.is_speech(frame_bytes, SAMPLE_RATE)
                if is_speech:
                    rms = calc_rms(frame_bytes)
                    if rms < 350.0:  # Filtra ruídos de fundo fracos ou zumbidos
                        is_speech = False
            except Exception:
                continue

            if not is_speaking:
                ring_buffer.append(frame_bytes)
                if is_speech:
                    speech_count += 1
                else:
                    speech_count = 0

                # Requer 3 frames consecutivos com voz e energia (90ms)
                if speech_count >= 3:
                    is_speaking = True
                    silence_count = 0
                    speech_frames.extend(ring_buffer)
                    ring_buffer.clear()
                    print("🗣️  [VOZ DETECTADA] Gravando sua fala...", file=sys.stderr, flush=True)
            else:
                speech_frames.append(frame_bytes)

                if not is_speech:
                    silence_count += 1
                else:
                    silence_count = 0

                if silence_count >= SILENCE_THRESHOLD_FRAMES:
                    if len(speech_frames) >= SPEECH_MIN_FRAMES:
                        print("✅ [FIM DA FALA] Processando transcrição...", file=sys.stderr, flush=True)
                        return speech_frames
                    else:
                        speech_frames.clear()
                        is_speaking = False
                        silence_count = 0
                        speech_count = 0
                        print("🎧 [ESCUTA CONTÍNUA] Aguardando sua voz...", file=sys.stderr, flush=True)
    finally:
        proc.terminate()
        proc.wait()


def save_wav(frames, path):
    with wave.open(path, 'wb') as wf:
        wf.setnchannels(1)
        wf.setsampwidth(2)
        wf.setframerate(SAMPLE_RATE)
        wf.writeframes(b''.join(frames))


def transcribe(audio_path):
    r = sr.Recognizer()
    try:
        with sr.AudioFile(audio_path) as source:
            audio = r.record(source)
            text = r.recognize_google(audio, language="pt-BR")
            return text.strip()
    except Exception:
        return ""


def main():
    try:
        frames = record_until_silence()
        if frames:
            save_wav(frames, AUDIO_OUTPUT_PATH)
            text = transcribe(AUDIO_OUTPUT_PATH)
            if text:
                print(text, flush=True)
                sys.exit(0)
    except KeyboardInterrupt:
        sys.exit(0)
    except Exception as e:
        print(f"Erro no listen_daemon: {e}", file=sys.stderr)
    sys.exit(1)


if __name__ == "__main__":
    main()
