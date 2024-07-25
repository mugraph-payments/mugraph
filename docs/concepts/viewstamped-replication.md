# Viewstamped Replication

## Introduction

Viewstamped Replication (VR) is a replication protocol that enables the creation of highly available distributed systems capable of tolerating crash failures. VR provides state machine replication, allowing clients to execute general operations to observe and modify the replicated service state. This protocol is suitable for implementing replicated services such as lock managers, file systems, or other stateful applications that require high availability.

This document describes the core VR protocol, including normal operation, view changes, and recovery. It also discusses key optimizations and practical considerations for implementing VR in real-world systems.

## Overview

VR replicates a service across a group of replica nodes. The service maintains state that is accessible to a set of client machines. To ensure reliability and availability, VR uses replica groups of size $2f+1$, where $f$ is the maximum number of faulty replicas the system can tolerate.

The protocol employs a primary replica to order client requests. Other replicas act as backups that accept the order selected by the primary. The system progresses through a sequence of views, with a different replica acting as primary in each view. If the primary appears faulty, the backups initiate a view change to select a new primary.

VR comprises three main sub-protocols:

1. Normal case processing of client requests
2. View changes to select a new primary 
3. Recovery of failed replicas

## Normal Operation

During normal operation, when the primary is not faulty and all participating replicas are in the same view, the protocol proceeds as follows:

1. The client sends a REQUEST message to the primary. This message contains:
   - The operation to be executed
   - The client's unique identifier
   - A monotonically increasing request number

2. Upon receiving the REQUEST, the primary:
   - Advances its op-number
   - Adds the request to its log
   - Updates the client table
   - Sends a PREPARE message to the other replicas

3. Backup replicas process PREPARE messages in order. For each message, a backup:
   - Adds the request to its log
   - Updates its client table
   - Sends a PREPAREOK message to the primary

4. The primary waits to receive $f$ PREPAREOK messages from different backups. Once received, it:
   - Considers the operation committed
   - Executes the operation
   - Sends a REPLY message to the client

5. The primary informs backups about commits, either:
   - In the next PREPARE message, or
   - Via separate COMMIT messages if no new requests arrive promptly

6. Upon learning of a commit, backups:
   - Execute the committed operation
   - Update their client tables

This process ensures that all replicas maintain consistent state and that client requests are processed in a well-defined order.

## View Changes

View changes allow the system to make progress when the primary fails. The protocol ensures that all committed operations survive into the new view in the same order. Key steps in the view change protocol include:

1. Backup replicas initiate a view change if they do not receive timely communication from the primary.

2. To start a view change, a replica:
   - Increments its view number
   - Sets its status to "view-change"
   - Sends STARTVIEWCHANGE messages to other replicas

3. Upon receiving $f$ STARTVIEWCHANGE messages for its new view number, a replica sends a DOVIEWCHANGE message to the node that will be the primary in the new view.

4. The new primary waits to receive $f+1$ DOVIEWCHANGE messages from different replicas (including itself). It then:
   - Sets its view number to that in the messages
   - Selects the most up-to-date log from the received messages
   - Updates its state based on this log
   - Sends STARTVIEW messages to other replicas

5. Upon receiving a STARTVIEW message, other replicas:
   - Update their logs and state based on the information in the message
   - Change their status to normal
   - Resume normal operation in the new view

This process ensures that the system can continue to make progress even when the primary fails, while maintaining consistency across view changes.

## Recovery

The recovery protocol allows failed replicas to rejoin the group with an up-to-date state:

1. A recovering replica sends RECOVERY messages to all other replicas. These messages include a nonce to uniquely identify the recovery attempt.

2. Active replicas respond with RECOVERYRESPONSE messages containing:
   - Their current view number
   - Their log (if they are the primary)
   - Their op-number and commit-number

3. The recovering replica waits for $f+1$ RECOVERYRESPONSE messages, including one from the current primary, all containing the nonce it sent.

4. Using the information from these responses, the recovering replica:
   - Updates its state to match that of the primary
   - Sets its status to normal
   - Joins the current view

This process ensures that recovered replicas rejoin the group with a consistent and up-to-date state.

## Optimizations

Several optimizations can significantly improve VR performance:

### Witnesses
Use $f$ witness replicas that do not store full state or execute operations. This reduces resource requirements while maintaining fault tolerance.

### Batching
Process multiple client requests in a single protocol round. This improves throughput, especially under high load. The throughput improvement can be modeled as:

$$
\text{Throughput}_{\text{batched}} = \frac{\text{Batch Size}}{\text{Protocol Round Time}} \quad \text{requests/second}
$$

### Fast Reads
Allow the primary to execute read-only operations without consulting other replicas. Use leases to ensure consistency, preventing stale reads after view changes. The lease duration $T$ should satisfy:

$$
T < \frac{\text{View Change Timeout}}{2}
$$

### Checkpoints
Periodically create snapshots of application state. This speeds up recovery and allows for log truncation, reducing storage requirements. The storage savings can be estimated as:

$$
\text{Storage Saved} = \text{Log Size} - \text{Checkpoint Size} - \text{Log Size Since Checkpoint}
$$

### Efficient Log Management
Keep a prefix of the log on disk and push updates to disk in the background. This reduces the cost of the recovery protocol and improves normal operation performance.

## Implementation Considerations

When implementing VR, consider the following:

- Use efficient data structures for the operation log and client table. For example, implement the client table using an in-memory key-value store.

- Implement proper concurrency control to handle simultaneous client requests and protocol messages. Use techniques like buffered channels or thread-safe queues to manage incoming requests.

- Design the system to gracefully handle network partitions and message reordering. Implement timeouts and retries for all network communications.

- Provide mechanisms for clients to locate the current primary, especially after view changes. Consider implementing a gossip protocol or using a centralized configuration service.

- Implement state transfer protocols to efficiently synchronize replica state. Use techniques like Merkle trees to minimize the amount of data transferred during recovery. The efficiency of Merkle trees can be expressed as:

$$
\text{Data Transferred} = O(\log N \cdot \text{Diff Size})
$$

  Where $N$ is the total number of state elements and Diff Size is the number of different elements between replicas.

- Carefully manage view numbers and operation numbers to ensure uniqueness and proper ordering across view changes. The relationship between view numbers and operation numbers can be expressed as:

$$
\text{Global Order} = \text{View Number} \cdot \text{Max Operations Per View} + \text{Operation Number}
$$

- Implement proper error handling and logging to facilitate debugging and system monitoring.

## Conclusion

Viewstamped Replication provides a robust foundation for building highly available distributed systems that can tolerate crash failures. By carefully implementing the core protocol and relevant optimizations, developers can create reliable replicated services that maintain consistency and availability in the face of node failures and network issues.

While VR offers strong consistency guarantees, it's important to consider the trade-offs between consistency, availability, and partition tolerance when designing distributed systems. For some applications, eventual consistency models or other replication strategies may be more appropriate.

As distributed systems continue to grow in importance, protocols like VR play a crucial role in ensuring the reliability and availability of critical services. Understanding and implementing these protocols correctly is essential for building robust, scalable distributed applications.