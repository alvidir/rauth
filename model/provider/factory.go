package provider

import (
	"fmt"
	"sync"
	"time"
)

type providerKey string
type providerValue Provider

var providers = &sync.Map{}

// New builds a brand new provider
func New(name string, timeout time.Duration) Provider {
	return &provider{
		Name:    name,
		Timeout: timeout,
	}
}

// AddProvider adds a new provider with the given name
func AddProvider(provider Provider) (err error) {
	if provider == nil {
		return fmt.Errorf(errNilProvider)
	}

	pName := providerKey(provider.GetName())
	if _, ok := providers.Load(pName); ok {
		return fmt.Errorf(errProviderAlreadyExists)
	}

	providers.Store(pName, providerValue(provider))
	return
}

// RemoveProvider removes the provider with the given name, if exists
func RemoveProvider(name string) {
	pName := providerKey(name)
	providers.Delete(pName)
}

// FindProvider finds the provider for a given name
func FindProvider(name string) (provider Provider, ok bool) {
	key := providerKey(name)

	var content interface{}
	if content, ok = providers.Load(key); ok {
		provider, ok = content.(providerValue)
	}

	return
}
