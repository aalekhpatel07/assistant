package cache

import (
	"bytes"
	"congo/pkg"
	"context"
	"errors"
	"fmt"
	"github.com/redis/go-redis/v9"
	"log"
	"os"
	"strconv"
)

type RedisCache struct {
	client *redis.Client
	ctx    context.Context
}

func NewRedisCache(ctx context.Context) (*RedisCache, error) {

	addr, exists := os.LookupEnv("REDIS_ADDR")
	if !exists {
		return nil, errors.New("could not find REDIS_ADDR")
	}
	password, exists := os.LookupEnv("REDIS_PASSWORD")
	if !exists {
		log.Printf("No REDIS_PASSWORD provided. Will use empty string.")
		password = ""
	}
	username, exists := os.LookupEnv("REDIS_USERNAME")
	if !exists {
		log.Printf("No REDIS_USERNAME provided. Will use empty string.")
		password = ""
	}
	db_, exists := os.LookupEnv("REDIS_DATABASE")
	if !exists {
		db_ = "0"
	}
	db, err := strconv.Atoi(db_)
	if err != nil {
		return nil, err
	}

	client := redis.NewClient(&redis.Options{
		Addr:     addr,
		Password: password,
		DB:       db,
		Username: username,
	})

	return &RedisCache{
		client: client,
		ctx:    ctx,
	}, nil
}

func (cache *RedisCache) Load(text string) (pkg.Option[*bytes.Buffer], error) {
	val, err := cache.client.Get(cache.ctx, fmt.Sprintf("text-to-speech:%s", text)).Bytes()
	if errors.Is(err, redis.Nil) {
		return pkg.None[*bytes.Buffer](), nil
	} else if err != nil {
		return pkg.None[*bytes.Buffer](), err
	}
	return pkg.Some[*bytes.Buffer](bytes.NewBuffer(val)), nil
}

func (cache *RedisCache) Store(text string, audio *bytes.Buffer) error {
	return cache.client.Set(
		cache.ctx,
		fmt.Sprintf("text-to-speech:%s", text),
		audio.Bytes(),
		60*60*24*14,
	).Err()
}
