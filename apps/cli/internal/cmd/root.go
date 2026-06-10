package cmd

import (
	"io"
	"os"

	"github.com/skkil/clip/cli/internal/services"
	"github.com/skkil/clip/cli/internal/tui"
	"github.com/spf13/cobra"
)

var root = &cobra.Command{
	Use:   "clip",
	Short: "A clipboard manager",
	Long:  "clip saves and manages clips. Pipe text to save a clip, or run with no arguments to open the TUI.",
	RunE:  run,
}

func run(cmd *cobra.Command, args []string) error {
	stat, _ := os.Stdin.Stat()
	if (stat.Mode() & os.ModeCharDevice) == 0 {
		data, err := io.ReadAll(os.Stdin)
		if err != nil {
			return err
		}

		svc := services.NewClipService()
		return svc.Save(string(data))
	}

	return tui.Run()
}

func init() {
	root.AddCommand(auth)
	root.AddCommand(copy)
}

func Execute() error {
	return root.Execute()
}
