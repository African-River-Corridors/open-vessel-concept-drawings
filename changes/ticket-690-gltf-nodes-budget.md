<!-- unreleased -->
- glTF and triangle budget (issue #690, spec 0009): each tank, dome, weather shield and Deck Item is a named glTF node (the raised and lowered wheelhouse are two), with the Light theme colour of its kind as its material. A test pins each fixture layer's triangle count and fails above 1.5 times it, or above 200,000 for any Design. No result value changes.
