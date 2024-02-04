package tts

import (
	"bytes"
	"errors"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"os"
	"time"
)

type Whisper struct {
	client  *http.Client
	baseUrl string
}

func NewWhisper() (*Whisper, error) {

	transport := http.Transport{
		ResponseHeaderTimeout: time.Duration(time.Second * 30),
	}

	client := http.Client{
		Transport: &transport,
	}

	baseUrl, exists := os.LookupEnv("WHISPER_BASE_URL")
	if !exists {
		return nil, errors.New("please provide WHISPER_BASE_URL to use Whisper")
	}

	return &Whisper{
		client:  &client,
		baseUrl: baseUrl,
	}, nil
}

func (w *Whisper) Speak(text string) (*bytes.Buffer, error) {

	params := url.Values{}
	params.Set("text", text)

	fullUrl := fmt.Sprintf("%s/api/tts?%s", w.baseUrl, params.Encode())
	resp, err := w.client.Get(fullUrl)
	if err != nil {
		return nil, err
	}

	if resp.StatusCode != http.StatusOK {
		return nil, errors.New(fmt.Sprintf("bad status: %s", resp.Status))
	}

	defer resp.Body.Close()
	contents, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}

	return bytes.NewBuffer(contents), nil
}
