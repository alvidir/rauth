package client

// A Extension represents a kind of client that extends from Client
type Extension interface {
	// GetName returns the client name
	GetName() string
}

// A Client represents a registered client
type Client interface {
	Extension

	GetID() string
	GetStatus() string
	IsPassword(string) bool
}
