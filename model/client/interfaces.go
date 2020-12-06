package client

// Extension represents a client extension
type Extension interface {
	GetAddr() string
}

// A Controller represents a registered client
type Controller interface {
	ClientStatus() Status
	MatchPassword(string) bool
	GetAddr() string
	GetID() int64
	SetExtension(Extension) bool
}
