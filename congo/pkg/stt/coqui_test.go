package stt

import (
	"bytes"
	audio2 "github.com/auroraapi/aurora-go/audio"
	"os"
	"testing"
)

func TestCoquiNew(t *testing.T) {
	prev, exists := os.LookupEnv("COQUI_BASE_URL")
	if err := os.Setenv("COQUIAI_BASE_URL", "http://localhost:9003"); err != nil {
		t.Error("could not set env var")
	}
	coqui, err := NewCoqui()
	if err != nil {
		t.Error("failed to create a new Coqui AI client")
	}
	if coqui.baseUrl != "http://localhost:9003" {
		t.Error("did not set the correct baseUrl")
	}
	if exists {
		if err := os.Setenv("COQUI_BASE_URL", prev); err != nil {
			t.Error("Failed to restore the COQUI_BASE_URL env var.")
		}
	}
}

func TestCoqui_Transcribe(t *testing.T) {

	prev, exists := os.LookupEnv("COQUI_BASE_URL")
	if err := os.Setenv("COQUI_BASE_URL", "http://localhost:9003"); err != nil {
		t.Error("could not set env var")
	}
	coqui, err := NewCoqui()
	if err != nil {
		t.Error("failed to create a new Coqui AI client")
	}

	recording, err := audio2.NewFileFromRecording(0.1, 0)
	if err != nil {
		t.Fatalf("%s", err)
	}
	contents := recording.WAVData()

	text, err := coqui.Transcribe(bytes.NewBuffer(contents))

	if err != nil {
		t.Fatalf("Failed to transcribe audio: %s", err)
	}
	t.Logf("transcription: %s", text)
	if exists {
		if err := os.Setenv("COQUI_BASE_URL", prev); err != nil {
			t.Error("Failed to restore the COQUI_BASE_URL env var.")
		}
	}
}
