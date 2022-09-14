package com.raphtory.examples.twitter.higgsdataset.graphbuilders

import com.raphtory.api.input.Graph
import com.raphtory.api.input.ImmutableProperty
import com.raphtory.api.input.Properties
import com.raphtory.api.input.Type
import com.raphtory.internals.graph.GraphBuilder

object TwitterGraphBuilder {

  def parser(graph: Graph, tuple: String): Unit = {
    val fileLine   = tuple.split(",").map(_.trim)
    val sourceNode = fileLine(0).trim
    val srcID      = sourceNode.toLong
    val targetNode = fileLine(1).trim
    val tarID      = targetNode.toLong
    val timeStamp  = fileLine(2).toLong

    graph.addVertex(timeStamp, srcID, Properties(ImmutableProperty("name", sourceNode)), Type("User"))
    graph.addVertex(timeStamp, tarID, Properties(ImmutableProperty("name", targetNode)), Type("User"))
    //Edge shows srcID retweets tarID's tweet
    graph.addEdge(timeStamp, srcID, tarID, Type("Retweet"))
  }
}