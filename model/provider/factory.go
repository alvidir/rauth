package provider

import (
	"fmt"
	"sync"
	"time"

	"github.com/alvidir/tp-auth/model/session"
)

type providerName string
type sessionName string

var providers = &sync.Map{}

// New builds a brand new provider
func New(name string, timeout time.Duration) Provider {
	return &provider{
		Name:     name,
		Timeout:  timeout,
		sessions: make(map[string]*session.Session),
	}
}

// AddProvider adds a new provider with the given name
func AddProvider(provider Provider) (err error) {
	if provider == nil {
		return fmt.Errorf(errNilProvider)
	}

	pName := providerName(provider.GetName())
	if _, ok := providers.Load(pName); ok {
		return fmt.Errorf(errProviderAlreadyExists)
	}

	providers.Store(pName, provider)
	return
}

// RemoveProvider removes the provider with the given name, if exists
func RemoveProvider(name string) {
	pName := providerName(name)
	providers.Delete(pName)
}

// FindProvider finds the provider for a given name
func FindProvider(name string) (provider Provider, ok bool) {
	providers.Range(func(key interface{}, value interface{}) bool {
		var currentName providerName
		if currentName, ok = key.(providerName); !ok {
			// ok is from FindProvider output not the Range's one, so keeping it as !ok means not found.
			// btw if assert it's true, ok
			return true
		}

		if ok = providerName(name) == currentName; ok {
			provider, ok = value.(Provider)
		}

		return !ok // while not found
	})

	return
}
