package stt

import (
	"bytes"
	"errors"
	"fmt"
	"io"
	"log"
	"mime/multipart"
	"net/http"
	"time"
)
import "os"

type Coqui struct {
	client  *http.Client
	baseUrl string
}

func NewCoqui() (*Coqui, error) {

	transport := http.Transport{
		ResponseHeaderTimeout: time.Duration(time.Second * 30),
	}

	client := http.Client{
		Transport: &transport,
	}

	baseUrl, exists := os.LookupEnv("COQUI_BASE_URL")
	if !exists {
		return nil, errors.New("please provide COQUI_BASE_URL to use COQUI")
	}

	return &Coqui{
		client:  &client,
		baseUrl: baseUrl,
	}, nil
}

func (c *Coqui) Transcribe(audio *bytes.Buffer) (*bytes.Buffer, error) {

	body := &bytes.Buffer{}
	writer := multipart.NewWriter(body)
	fw, err := writer.CreateFormFile("audio_file", "dummy.wav")
	if err != nil {
		return nil, err
	}
	_, err = io.Copy(fw, audio)
	if err != nil {
		return nil, err
	}

	formData := map[string]string{
		"encode":          "true",
		"task":            "transcribe",
		"word_timestamps": "false",
		"output":          "txt",
		"language":        "en",
	}

	for key, value := range formData {
		field, err := writer.CreateFormField(key)
		if err != nil {
			return nil, err
		}
		bytesWritten, err := field.Write([]byte(value))
		if err != nil {
			return nil, err
		}
		if bytesWritten != len(value) {
			return nil, errors.New(fmt.Sprintf("could not write field %s value completely", key))
		}
	}

	if err := writer.Close(); err != nil {
		return nil, err
	}

	request, err := http.NewRequest("POST", fmt.Sprintf("%s/asr", c.baseUrl), bytes.NewReader(body.Bytes()))
	if err != nil {
		return nil, err
	}
	request.Header.Set("Content-Type", writer.FormDataContentType())

	resp, err := c.client.Do(request)
	if err != nil {
		return nil, err
	}

	if resp.StatusCode != http.StatusOK {
		contents, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("bad status: %s: %s", resp.Status, contents)
	}

	contents, err := io.ReadAll(resp.Body)
	defer func() {
		err := resp.Body.Close()
		if err != nil {
			log.Panic("failed to close body")
		}
	}()
	return bytes.NewBuffer(contents), nil
}
