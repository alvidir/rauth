package app

// A Controller represents an App client
type Controller interface {
	GetDescription() string
	GetName() string
	GetURL() string
}
