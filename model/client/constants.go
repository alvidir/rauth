package client

// Status represents the Status enum
type Status int

// Status possible values
const (
	errCredentialAlreadyExists = "Credential %s already exists"

	PENDING Status = iota
	ACTIVATED
	DEACTIVATED
)

func (s Status) String() string {
	return [...]string{
		"Pending",
		"Activated",
		"Deactivated",
	}[s]
}
