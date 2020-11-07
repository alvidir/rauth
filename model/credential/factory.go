package credential

// NewCredential builds a brand new credetial for a given public key
func NewCredential(key string) *Credential {
	return &Credential{Public: key}
}
