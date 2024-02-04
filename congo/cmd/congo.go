package main

import (
	"bytes"
	"congo/pkg"
	"congo/pkg/cache"
	"congo/pkg/llm"
	"congo/pkg/stt"
	"congo/pkg/tts"
	"context"
	"fmt"
	"github.com/auroraapi/aurora-go/audio"
	"github.com/briandowns/spinner"
	"log"
	"time"
)

type Driver struct {
	stt   pkg.SpeechToText
	tts   pkg.TextToSpeech
	llm   pkg.Converse
	cache pkg.TextToSpeechCache
}

func (d *Driver) Run() {

	listeningSpinner := spinner.New(spinner.CharSets[11], 100*time.Millisecond)
	listeningSpinner.Suffix = "\t Listening..."
	_ = listeningSpinner.Color("blue")

	transcribingSpinner := spinner.New(spinner.CharSets[11], 100*time.Millisecond)
	transcribingSpinner.Suffix = "\t Processing..."
	_ = transcribingSpinner.Color("yellow")

	conversingSpinner := spinner.New(spinner.CharSets[11], 100*time.Millisecond)
	conversingSpinner.Suffix = "\t Thinking..."
	_ = conversingSpinner.Color("green")

	convertingToSpeechSpinner := spinner.New(spinner.CharSets[11], 100*time.Millisecond)
	convertingToSpeechSpinner.Suffix = "\t Converting to audio..."
	_ = convertingToSpeechSpinner.Color("yellow")

	speakingSpinner := spinner.New(spinner.CharSets[11], 100*time.Millisecond)
	speakingSpinner.Suffix = "\t Speaking..."
	_ = speakingSpinner.Color("yellow")

	for {
		listeningSpinner.Start()
		audioReader, err := audio.NewFileFromRecording(0, 1)
		listeningSpinner.Stop()
		if err != nil {
			log.Printf("%s", err)
			continue
		}
		wav := audioReader.WAVData()
		log.Printf("recorded %d bytes\n", len(wav))
		contents := bytes.NewBuffer(wav)

		transcribingSpinner.Start()
		transcription, err := d.stt.Transcribe(contents)
		transcribingSpinner.Stop()
		if err != nil {
			fmt.Printf("%s\n", err)
			continue
		}
		text := string(transcription.Bytes())
		if text == "" {
			continue
		}
		fmt.Printf("Transcribed: %s\n", text)

		conversingSpinner.Start()
		resp, err := d.llm.Converse(text)
		conversingSpinner.Stop()
		if err != nil {
			fmt.Printf("Failed to get response from LLM: %s\n", err)
			resp = "could not get response from llama."
		}
		if resp == "" {
			resp = "could not get response from llama."
		}
		fmt.Printf("response from llm: %s", resp)

		var audioData *bytes.Buffer

		convertingToSpeechSpinner.Start()
		cachedAudio, err := d.cache.Load(resp)

		if err == nil && cachedAudio.IsSome() {
			audioData = cachedAudio.Unwrap()
		} else {
			if err != nil {
				fmt.Printf("Cache possibly broken: %s", err)
			}
			audioData, err = d.tts.Speak(resp)
			if err := d.cache.Store(text, audioData); err != nil {
				fmt.Printf("Failed to cache...")
			}
		}

		convertingToSpeechSpinner.Stop()
		if err != nil {
			fmt.Printf("%s\n", err)
			continue
		}

		file, err := audio.NewFileFromBytes(audioData.Bytes())
		if err != nil {
			fmt.Printf("%s\n", err)
			continue
		}
		speakingSpinner.Start()
		if err := file.Play(); err != nil {
			fmt.Printf("failed to play: %s", err)
			speakingSpinner.Stop()
			continue
		}
		speakingSpinner.Stop()
	}
}

func NewDriver(stt pkg.SpeechToText, tts pkg.TextToSpeech, cache pkg.TextToSpeechCache, llm pkg.Converse) *Driver {
	return &Driver{
		stt:   stt,
		tts:   tts,
		cache: cache,
		llm:   llm,
	}
}

func main() {
	ctx := context.Background()
	speechToText, err := stt.NewCoqui()
	if err != nil {
		log.Fatalf("Failed to create SpeechToText client: %s", err)
	}
	textToSpeech, err := tts.NewWhisper()
	if err != nil {
		log.Fatalf("Failed to create TextToSpeech client: %s", err)
	}
	redisCache, err := cache.NewRedisCache(ctx)
	phi, err := llm.NewPhi()
	driver := NewDriver(speechToText, textToSpeech, redisCache, phi)
	driver.Run()
}
