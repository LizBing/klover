/*
 * Copyright (c) 2024, Lei Zaakjyu. All rights reserved.
 *
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

#ifndef UTIL_ALIGN_H
#define UTIL_ALIGN_H

#define align_down(n, a) ((n) & ~(a - 1))
#define align_up(n, a) (align_down((n) + a - 1, a))

#define alignment_mask(a) (a - 1)

#define is_aligned(n, a) ((n & alignment_mask(a)) == 0)
  

#endif // UTIL_ALIGN_H

