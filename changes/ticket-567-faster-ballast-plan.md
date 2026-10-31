<!-- unreleased -->
- Speed (issue #567): loading conditions with the same mass and centre of gravity now solve each heel of the GZ curve once, and the hull cut reuses the section at the end of one sub-interval as the start of the next. The default ShipD Design in the IS Code 2008 conditions runs in about 0.43 s instead of 1.2 s (release build). No result changes: every results hash is the same.
