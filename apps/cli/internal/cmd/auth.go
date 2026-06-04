package cmd

import (
	"fmt"

	"github.com/spf13/cobra"
)

var auth = &cobra.Command{
	Use:   "auth",
	Short: "Authenticate with the clip service",
	RunE: func(cmd *cobra.Command, args []string) error {
		fmt.Println("auth: not yet implemented")
		return nil
	},
}
