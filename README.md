# Confidentiality-preserving computationg using homomorphic encryption

## How to run?

This repository has two ways of running the working example:

1. Ansible deployment,
2. Manual deployment.

Each of these are going to be explained bellow.

### Ansible deployment

The Ansible deployment is the most easy one to run. All you have to do, is to run

```
ansible-playbook run.yml
```

which will build and deploy all of the Docker container in a Docker Compose.

Unfortunatyle, this takes way too much time, since you will have to build the images from scratch, and thus the other method, the **manual deployment** is ***recommended***.

### Manual deployment

The manual deployment has is split into two parts.

The first part is the **backend**, where you have to navigate to `backend` and run

```
cargo run --release
```

The second part is the **frontned**, where you have to navigate to `frontned` and run

```
npm start
```
## How it works?

The best way to know how it works, is to read the documentation, meaning the *code* itself, and also play around with the `backend` and `frontend`, by examining the different calls.
