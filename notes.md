compute areas of high resource density over a suitable parameterized area size

loop
	update resource density each frame
	queue areas in decreasing density resource
	      if previously assigned areas have resource below a threshold, get a new area
	      	 distribute high priorty areas to groups of agents by assigning these areas to agents
	let agents work and collect from assigned areas
