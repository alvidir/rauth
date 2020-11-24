package user

// A Controller represents an user client
type Controller interface {
	GetNickname() string
	GetEmail() string
}
