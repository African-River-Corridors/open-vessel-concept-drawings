<!-- unreleased -->
- Damage stability (issue #755, spec 0012): the solver floods Flooded Volumes, not boxes. A Flooded Volume is a Space shape (a prism or a Type C tank) with a permeability and a list of Intact Inclusions, Spaces inside it that keep their buoyancy, such as an intact tank in a flooded hold. A box is the prism with four corners, so no liquid result moves.
