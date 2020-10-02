package provider

import (
	"fmt"
	"sync"
)

type providerName string

var providers = &sync.Map{}

// AddProvider adds a new provider with the given name
func AddProvider(name string, provider Provider) (err error) {
	if provider == nil {
		return fmt.Errorf(errNilProvider)
	}

	pName := providerName(name)
	if _, ok := providers.Load(pName); !ok {
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

		providerName := providerName(name)
		if ok = providerName == currentName; ok {
			provider = value
		}

		return !ok // while not found
	})

	return
}
