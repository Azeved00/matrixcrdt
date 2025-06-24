from invoke import Collection
from . import macro 

ns = Collection()
ns.add_collection(Collection.from_module(macro), name="macro")
