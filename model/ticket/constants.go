package ticket

// Kind represents the ticket's kind
type Kind int

// Kind possible values
const (
	ACTIVATION Kind = iota
	DEACTIVATION
)

func (kind Kind) String() string {
	return [...]string{
		"Activation",
		"Deactivation",
	}[kind]
}
