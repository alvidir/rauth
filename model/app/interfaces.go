package app

// A Controller represents an App client
type Controller interface {
	GetName() string
	GetURL() string
}
