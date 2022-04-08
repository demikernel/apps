Demo Applications for Demikernel
================================

[![Join us on Slack!](https://img.shields.io/badge/chat-on%20Slack-e01563.svg)](https://join.slack.com/t/demikernel/shared_invite/zt-11i6lgaw5-HFE_IAls7gUX3kp1XSab0g)
[![Build](https://github.com/demikernel/apps/actions/workflows/build.yml/badge.svg)](https://github.com/demikernel/apps/actions/workflows/build.yml)

This repository various demo applications for Demikernel:
- [x] `tcp-pktgen`: Generates TCP Packets
- [x] `udp-dump`: Dumps Incoming Packets on a UDP Port
- [x] `udp-echo`: Echoes UDP Packets
- [x] `udp-pktgen`: Generates UDP Packets
- [x] `udp-relay`: Relays UDP Packets

Building
---------

**1. Install Prerequisites**
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh    # Get Rust toolchain.
```

**2. Clone This Repository**
```
export WORKDIR=$HOME                               # Change this to whatever you want.
cd $WORKDIR                                        # Switch to working directory.
git clone https://github.com/demikernel/apps.git   # Clone.
cd $WORKDIR/apps                                   # Switch to working directory.
```

**3. Build This Utility**
```
# Build for Catnap LibOS
make all LIBOS=catnap

# Build for Catnip LibOS
make all LIBOS=catnip

# Build for Catpowder LibOS
make all LIBOS=catpowder
```

Code of Conduct
---------------

This project has adopted the [Microsoft Open Source Code of Conduct](https://opensource.microsoft.com/codeofconduct/).
For more information see the [Code of Conduct FAQ](https://opensource.microsoft.com/codeofconduct/faq/)
or contact [opencode@microsoft.com](mailto:opencode@microsoft.com) with any additional questions or comments.

Usage Statement
--------------

This project is a prototype. As such, we provide no guarantees that it will
work and you are assuming any risks with using the code. We welcome comments
and feedback. Please send any questions or comments to one of the following
maintainers of the project:

- [Irene Zhang](https://github.com/iyzhang) - [irene.zhang@microsoft.com](mailto:irene.zhang@microsoft.com)
- [Pedro Henrique Penna](https://github.com/ppenna) - [ppenna@microsoft.com](mailto:ppenna@microsoft.com)

> By sending feedback, you are consenting that it may be used  in the further
> development of this project.
