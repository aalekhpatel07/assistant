package pkg

import (
	"bytes"
)

type SpeechToText interface {
	Transcribe(audio *bytes.Buffer) (*bytes.Buffer, error)
}

type TextToSpeech interface {
	Speak(text string) (*bytes.Buffer, error)
}

type Option[T any] struct {
	value    T
	occupied bool
}

func Some[T any](value T) Option[T] {
	return Option[T]{
		value:    value,
		occupied: true,
	}
}

func None[T any]() Option[T] {
	return Option[T]{occupied: false}
}

func (s *Option[T]) Unwrap() T {
	if s.occupied {
		return s.value
	} else {
		panic("cannot Unwrap a None value")
	}
}

func (s *Option[T]) IsSome() bool {
	return s.occupied
}

type TextToSpeechCache interface {
	Load(text string) (Option[*bytes.Buffer], error)
	Store(text string, audio *bytes.Buffer) error
}

type Converse interface {
	Converse(text string) (string, error)
}
