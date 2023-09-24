#!/usr/bin/env python3

import os
from N2G import drawio_diagram


class UML_Generator:
	def __init__(self, srcdir, outdir, subdir=''):
		if len(subdir) > 1:
			self.src_dir = os.path.join(srcdir, subdir)
		else:
			self.src_dir = srcdir

		self.prj_dir = srcdir
		self.out_dir = outdir
		self.prj_name = srcdir.split('/')[-1]

		self.nodes = []
		self.edges = []
		self.graph = {}

	def has_node(self, label):
		for node in self.nodes:
			if node['label'] == label:
				return True

		return False

	def has_edge(self, source, target):
		for edge in self.edges:
			if edge['source'] == source and edge['target'] == target:
				return True

		return False

	def dump_nodes(self):
		print('Nodes:')
		for node in self.nodes:
			print(f"\t{node['label']}")

	def dump_edges(self):
		print('Edges:')
		for edge in self.edges:
			print(f"\t{edge['source']} --{edge['label']}--> {edge['target']}")

	def get_nodes(self):
		for root, subdirs, files in os.walk(self.src_dir):
			for file in files:
				node = file.split('.')[0]
				if not self.has_node(node):
					self.nodes.append({'id': node, 'label': node, 'width': 85, 'height': 50})
		self.dump_nodes()

	def get_edges(self):
		for root, subdirs, files in os.walk(self.src_dir):
			for file in files:
				node = file.split('.')[0]
				with open(os.path.join(root, file), 'r') as f:
					lines = f.readlines()
					for line in lines:
						if '#include' == line[:8]:
							include = line.split(' ')[1].strip('"<>\n')
							if '/' in include:
								include = include.split('/')[-1]

							source = include.split('.')[0]
							if not source == node:
								if self.has_node(node) and self.has_node(source):
									self.edges.append({'source' : source, 
														'label' : file.split('.')[1], 
														'target' : node, 
														'style' : 'Line End=1'})

		self.dump_edges()

	def make_graph(self):
		self.graph['nodes'] = self.nodes
		self.graph['edges'] = self.edges
		diagram = drawio_diagram()
		diagram.from_dict(self.graph, width=600, height=400)
		diagram.layout(algo="grid")
		diagram.dump_file(filename=f"{self.prj_name}_uml.drawio", folder=self.out_dir)
		
	def generate(self):
		self.get_nodes()
		self.get_edges()
		self.make_graph()					
