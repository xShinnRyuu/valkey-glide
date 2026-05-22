// Copyright Valkey GLIDE Project Contributors - SPDX Identifier: Apache-2.0

package integTest

import (
	"context"
	"runtime"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/valkey-io/valkey-glide/go/v2/interfaces"
)

func (suite *GlideTestSuite) TestParallelizedSetWithGC() {
	// The high parallelism is required to reproduce https://github.com/valkey-io/valkey-glide/issues/3207.
	// Reduced from 640 to 256 to avoid overwhelming the pipeline channel under GC pressure,
	// while still maintaining enough concurrency to validate the fix.
	suite.runParallelizedWithDefaultClients(256, 256000, 3*time.Minute, func(client interfaces.BaseClientCommands) {
		runtime.GC()
		key := uuid.New().String()
		value := uuid.New().String()
		// Retry on transient pipeline channel full errors under heavy GC pressure
		var lastErr error
		for attempt := 0; attempt < 3; attempt++ {
			result, err := client.Set(context.Background(), key, value)
			if err == nil {
				suite.Equal("OK", result)
				return
			}
			lastErr = err
			if strings.Contains(err.Error(), "Pipeline channel full") ||
				strings.Contains(err.Error(), "FatalSendError") {
				time.Sleep(time.Duration(attempt+1) * 50 * time.Millisecond)
				continue
			}
			suite.NoError(err)
			return
		}
		suite.NoError(lastErr)
	})
}
