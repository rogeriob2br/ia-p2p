import sys
import speech_recognition as sr

def main():
    audio_path = sys.argv[1] if len(sys.argv) > 1 else "/tmp/mic_input.wav"
    r = sr.Recognizer()
    try:
        with sr.AudioFile(audio_path) as source:
            audio = r.record(source)
            text = r.recognize_google(audio, language="pt-BR")
            print(text.strip())
    except sr.UnknownValueError:
        # Google não conseguiu reconhecer fala no áudio
        print("", end="")
    except sr.RequestError as e:
        # Erro de rede ou API
        print(f"Erro de rede na API de transcrição: {e}", file=sys.stderr)
        print("", end="")
    except Exception as e:
        print(f"Erro inesperado: {e}", file=sys.stderr)
        print("", end="")

if __name__ == "__main__":
    main()
