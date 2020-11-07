package namespace

import (
	"sync"
	"time"
)

type entry struct {
	Value     interface{} `json:"value" bson:"value"`
	UpdatedAt time.Time   `json:"updated_at" bson:"updated_at"`
	CreatedAt time.Time   `json:"created_at" bson:"created_at"`
}

// A Namespace represents a client's memory space
type Namespace struct {
	Entries map[string]entry `json:"entries" bson:"entries"`
	mu      sync.RWMutex
}

func (np *Namespace) Read(entry string) (value interface{}, err error) {
	np.mu.RLock()
	defer np.mu.RUnlock()

	return
}

func (np *Namespace) Write(entry string, value interface{}) (err error) {
	np.mu.Lock()
	defer np.mu.Unlock()

	return
}
