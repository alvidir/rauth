package client

// Status represents the Status enum
type Status int

// Status possible values
const (
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
