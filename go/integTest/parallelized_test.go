// Copyright Valkey GLIDE Project Contributors - SPDX Identifier: Apache-2.0

package integTest

import (
	"context"
	"errors"
	"runtime"
	"time"

	"github.com/google/uuid"
	glide "github.com/valkey-io/valkey-glide/go/v2"
	"github.com/valkey-io/valkey-glide/go/v2/interfaces"
)

func (suite *GlideTestSuite) TestParallelizedSetWithGC() {
	// The insane 640 parallelism is required to reproduce https://github.com/valkey-io/valkey-glide/issues/3207.
	suite.runParallelizedWithDefaultClients(640, 640000, 2*time.Minute, func(client interfaces.BaseClientCommands) {
		runtime.GC()
		key := uuid.New().String()
		value := uuid.New().String()

		var result string
		var err error
		// Under extreme parallelism with GC pressure, transient DisconnectErrors can occur
		// when the pipeline channel fills up. Retry since the client will reconnect.
		for attempt := 0; attempt < 3; attempt++ {
			result, err = client.Set(context.Background(), key, value)
			if err == nil {
				break
			}
			var discErr *glide.DisconnectError
			if !errors.As(err, &discErr) {
				break
			}
		}
		suite.verifyOK(result, err)
	})
}
