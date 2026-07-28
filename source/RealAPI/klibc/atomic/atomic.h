/*	$NetBSD: atomic.h,v 1.13 2015/01/08 22:27:18 riastradh Exp $	*/

/*-
 * Copyright (c) 2007, 2008 The NetBSD Foundation, Inc.
 * All rights reserved.
 *
 * This code is derived from software contributed to The NetBSD Foundation
 * by Jason R. Thorpe.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE NETBSD FOUNDATION, INC. AND CONTRIBUTORS
 * ``AS IS'' AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED
 * TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL THE FOUNDATION OR CONTRIBUTORS
 * BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
 * POSSIBILITY OF SUCH DAMAGE.
 */

#ifndef __realix_atomic_h__
#define	__realix_atomic_h__

#include "../stdint.h"
#include "../stddef.h"

#ifdef __cplusplus
extern "C" {
#endif 

typedef enum {
	MemoryAttrRelaxed 				= __ATOMIC_RELAXED,
	MemoryAttrConsume 				= __ATOMIC_CONSUME,
	MemoryAttrAcquire 				= __ATOMIC_ACQUIRE,
	MemoryAttrRelease 				= __ATOMIC_RELEASE,
	MemoryAttrAcquireAndRelease  	= __ATOMIC_ACQ_REL,
	MemoryAttrSeqAndCst  			= __ATOMIC_SEQ_CST
} AtomicMemoryAttributes;

#define AtomicLoadExplicit(object, order) \
	__atomic_load_n(object, order)

#define AtomicStoreExplicit(object, desired, order) \
	__atomic_store_n(object, desired, order)

#define AtomicExchangeExplicit(object, desired, order) \
	__atomic_exchange_n(object, desired, order)

#define AtomicCompareExchangeStrongExplicit(object, expected, desired, success, failure) \
	__atomic_compare_exchange_n(object, expected, desired, 0, success, failure)

void		AtomicBytePush(volatile uint8_t *object, int8_t delta);
void		Atomic16Push(volatile uint16_t *object, int16_t delta);
void		Atomic32Push(volatile uint32_t *object, int32_t delta);
void		AtomicPtrPush(volatile void **object, ptrdiff_t delta);

uint8_t		AtomicBytePushNewVal(volatile uint8_t *object, int8_t delta);
uint16_t	Atomic16PushNewVal(volatile uint16_t *object, int16_t delta);
uint32_t	Atomic32PushNewVal(volatile uint32_t *object, int32_t delta);
void *		AtomicPtrPushNewVal(volatile void **object, ptrdiff_t delta);

void		AtomicByteAnd(volatile uint8_t *object, uint8_t mask);
void		Atomic16And(volatile uint16_t *object, uint16_t mask);
void		Atomic32And(volatile uint32_t *, uint32_t);

uint8_t		AtomicByteAndNewVal(volatile uint8_t *object, uint8_t mask);
uint16_t	Atomic16AndNewVal(volatile uint16_t *object, uint16_t mask);
uint32_t	Atomic32AndNewVal(volatile uint32_t *object, uint32_t mask);

void		AtomicByteOr(volatile uint8_t *object, uint8_t mask);
void		Atomic16Or(volatile uint16_t *object, uint16_t mask);
void		Atomic32Or(volatile uint32_t *object, uint32_t mask);

uint8_t		AtomicByteOrNewVal(volatile uint8_t *object, uint8_t mask);
uint16_t	Atomic16OrNewVal(volatile uint16_t *object, uint16_t mask);
uint32_t	Atomic32OrNewVal(volatile uint32_t *object, uint32_t mask);

uint8_t 	AtomicByteCas(volatile uint8_t *object, uint8_t expected, uint8_t desired);
uint16_t	Atomic16Cas(volatile uint16_t *object, uint16_t expected, uint16_t desired);
uint32_t	Atomic32Cas(volatile uint32_t *object, uint32_t expected, uint32_t desired);
void *		AtomicPtrCas(volatile void **object, void *expected, void *desired);

uint8_t		NonInterlockedAtomicByteCas(volatile uint8_t *object, uint8_t expected, uint8_t desired);
uint16_t	NonInterlockedAtomic16Cas(volatile uint16_t *object, uint16_t expected, uint16_t desired);
uint32_t	NonInterlockedAtomic32Cas(volatile uint32_t *object, uint32_t expected, uint32_t desired);
void *		NonInterlockedAtomicPtrCas(volatile void **object, void *expected, void *desired);

uint8_t		AtomicByteSwap(volatile uint8_t *object, uint8_t value);
uint16_t	Atomic16Swap(volatile uint16_t *object, uint16_t value);
uint32_t	Atomic32Swap(volatile uint32_t *object, uint32_t value);
void *		AtomicPtrSwap(volatile void **object, void *value);

void		AtomicByteDecrement(volatile uint8_t *object);
void		Atomic16Decrement(volatile uint16_t *object);
void		Atomic32Decrement(volatile uint32_t *object);
void		AtomicPtrDecrement(volatile void **object);

uint8_t		AtomicByteDecrementNewVal(volatile uint8_t *object);
uint16_t	Atomic16DecrementNewVal(volatile uint16_t *object);
uint32_t	Atomic32DecrementNewVal(volatile uint32_t *object);
void *		AtomicPtrDecrementNewVal(volatile void **object);

void		AtomicByteIncrement(volatile uint8_t *object);
void		Atomic16Increment(volatile uint16_t *object);
void		Atomic32Increment(volatile uint32_t *object);
void		AtomicPtrIncrement(volatile void **object);

uint8_t		AtomicByteIncrementNewVal(volatile uint8_t *object);
uint16_t	Atomic16IncrementNewVal(volatile uint16_t *object);
uint32_t	Atomic32IncrementNewVal(volatile uint32_t *object);
void *		AtomicPtrIncrementNewVal(volatile void **object);

void	 	MemBarrierEnter(void);
void		MemBarrierExit(void);
void		MemBarrierProducer(void);
void		MemBarrierConsumer(void);
void		MemBarrierSync(void);

#ifdef	__HAVE_MEMBARRIER_DATADEPENDENCY_CONSUMER
void		MemBarrierDataDependencyConsumer(void);
#else
#define	MemBarrierDataDependencyConsumer()	((void)0)
#endif

#ifdef __cplusplus
}
#endif 

#endif /* ! __realix_atomic_h__ */