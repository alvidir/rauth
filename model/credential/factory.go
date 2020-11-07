package credential

// New builds a brand new credetial for a given public key
func New(key string) *Credential {
	return &Credential{Public: key}
}
