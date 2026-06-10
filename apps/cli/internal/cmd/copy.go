package cmd

import (
	"fmt"

	"github.com/spf13/cobra"
)

var copy = &cobra.Command{
	Use:   "copy",
	Short: "Copy a clip to the clipboard",
	RunE: func(cmd *cobra.Command, args []string) error {
		fmt.Println("copy: not yet implemented")
		return nil
	},
}
