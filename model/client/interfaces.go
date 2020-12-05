package client

// Extension represents a client extension
type Extension interface {
}

// A Controller represents a registered client
type Controller interface {
	GetStatus() string
	MatchPassword(string) bool
}
