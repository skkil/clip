package clip

import "fmt"

type Service struct{}

func NewService() *Service {
	return &Service{}
}

func (s *Service) Save(text string) error {
	fmt.Printf("saving clip: %q\n", text)
	return nil
}
