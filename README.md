# LIF_Server

A library for using space tuples in fog computing

The library offers the possibility to create servers with UDP and TCP protocols. These servers allow access to a repository that stores the tuples spaces. The primitives are based on the [Rustupolis library](https://github.com/micutio/rustupolis)

Voici la liste des différentes commandes disponibles : 

```rust
create {creation_attribute} {tuple_space_name} {permission_attribute}
create {creation_attribute} {tuple_space_name} {read_permission_attribute} {in_permission_attribute} {out_permission_attribute} {delete_permission_attribute}
delete {delete_permission_attribute} {tuple_space_name}
attach {tuple_space_name} {permission_attribute}*
out {tuple}    
out {tuple}(,{tuple})*    
read {tuple}    
read {tuple} (,{tuple})*
in {tuple} 
in {tuple} (,{tuple})*
```

An example for launching 2 servers is available in the file ```\example```

# Milestones

- [x] Make the tuple space available on the network
- [x] Add a access control system
- [x] Add the encryption on the communication
- [ ] Add a data persistency system
- [ ] Add a system of data placement policy 
