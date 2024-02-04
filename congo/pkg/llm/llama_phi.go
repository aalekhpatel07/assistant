package llm

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"os"
	"time"
)

type Phi struct {
	client  *http.Client
	baseUrl string
	history []string
}

func NewPhi() (*Phi, error) {
	transport := http.Transport{
		ResponseHeaderTimeout: time.Duration(time.Second * 30),
	}

	client := http.Client{
		Transport: &transport,
	}

	baseUrl, exists := os.LookupEnv("LLAMA_PHI_BASE_URL")
	if !exists {
		return nil, errors.New("please provide LLAMA_PHI_BASE_URL")
	}

	return &Phi{
		client:  &client,
		baseUrl: baseUrl,
	}, nil
}

func (c *Phi) buildMessageHistory() []map[string]string {
	messages := make([]map[string]string, 0)
	for idx, msg := range c.history {
		if idx%2 == 0 {
			messages = append(messages, map[string]string{
				"role":    "system",
				"content": msg,
			})
		} else {
			messages = append(messages, map[string]string{
				"role":    "user",
				"content": msg,
			})
		}
	}
	return messages
}

type completionResponse struct {
	Choices []completionChoice `json:"choices"`
}

type completionChoice struct {
	Index        int               `json:"index"`
	Message      completionMessage `json:"message"`
	FinishReason string            `json:"finish_reason"`
}

type completionMessage struct {
	Content string `json:"content"`
	Role    string `json:"role"`
}

func (c *Phi) Converse(text string) (string, error) {
	if text == "" {
		return "", nil
	}
	fullUrl := fmt.Sprintf("%s/v1/chat/completions", c.baseUrl)
	c.history = append(c.history, text)

	bodyObj := make(map[string][]map[string]string)
	bodyObj["messages"] = c.buildMessageHistory()

	body, err := json.Marshal(bodyObj)
	fmt.Printf("body: %s\n", body)
	if err != nil {
		return "", err
	}
	resp, err := c.client.Post(fullUrl, "application/json", bytes.NewBuffer(body))
	if err != nil {
		return "", err
	}
	if resp.StatusCode != http.StatusOK {
		fmt.Printf("%s", resp.Body)
		defer resp.Body.Close()
		return "", errors.New(fmt.Sprintf("bad status: %s", resp.Status))
	}
	defer func() {
		if err := resp.Body.Close(); err != nil {
			fmt.Printf("failed to close response body.")
		}
	}()

	contents, err := io.ReadAll(resp.Body)
	fmt.Printf("raw response: %s\n", contents)
	if err != nil {
		return "", err
	}

	var completionResp completionResponse
	if err := json.Unmarshal(contents, &completionResp); err != nil {
		fmt.Printf("failed to unmarshall %s", contents)
		return "", err
	}
	if len(completionResp.Choices) > 0 && completionResp.Choices[0].Message.Content != "" {
		c.history = append(c.history, completionResp.Choices[0].Message.Content)
		return completionResp.Choices[0].Message.Content, nil
	}
	return "", nil
}
