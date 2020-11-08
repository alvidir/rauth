package credential

// A Controller represents a public ssh key
type Controller interface {
	GetID() string
	GetName() string
	GetPublic() string
}
