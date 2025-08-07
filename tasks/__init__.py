from invoke import Collection

from . import macro 
from . import micro

ns = Collection()
ns.add_collection(Collection.from_module(macro), name="macro")
ns.add_collection(Collection.from_module(micro), name="micro")
