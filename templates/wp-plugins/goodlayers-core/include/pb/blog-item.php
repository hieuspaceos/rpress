<?php
	/*	
	*	Goodlayers Blog Item
	*/
	
	if( !class_exists('gdlr_core_blog_item') ){
		class gdlr_core_blog_item{
			
			var $settings = '';
			
			// init the variable
			function __construct( $settings = array() ){
				
				$this->settings = wp_parse_args($settings, array(
					'category' => '', 
					'tag' => '', 
					'num-fetch' => '9', 
					'layout' => 'fitrows',
					'prepend-sticky' => 'disable', 
					'show-thumbnail' => 'enable', 
					'thumbnail-size' => 'full', 
					'orderby' => 'date', 
					'order' => 'desc',
					'blog-style' => 'blog-full', 
					'has-column' => 'no',
					'no-space' => 'no',
					'excerpt' => 'specify-number', 
					'excerpt-number' => 55, 
					'column-size' => 20,
					'item-size' => 'small',
					'filterer' => 'none',
					'filterer-align' => 'center',
					'show-read-more' => 'enable',
					'meta-option' => array(),
					'pagination' => 'none'
				));

				if( $this->settings['orderby'] == 'rand' ){
					$this->settings['orderby'] = 'RAND(' . rand(1, 1000) . ')';
				} 
			}
			
			// get the content of the blog item
			function get_content(){
				
				if( !empty($this->settings['column-size']) ){
					gdlr_core_set_container_multiplier(intval($this->settings['column-size']) / 60, false);
				}

				$ret = '';
				if( empty($this->settings['query']) ){
					$query = $this->get_blog_query();
				}else{
					$query = $this->settings['query'];
				}
				
				if( $this->settings['blog-style'] == 'blog-feature' ){

					gdlr_core_setup_admin_postdata();

					$ret .= '<div class="gdlr-core-blog-item-holder clearfix" >';
					$ret .= $this->get_blog_feature_content($query);
					$ret .= '</div>';

					wp_reset_postdata();
					gdlr_core_reset_admin_postdata();

				// carousel style
				}else if( $this->settings['layout'] == 'carousel' ){
					$slides = array();
					$column_no = 60 / intval($this->settings['column-size']);

					$flex_atts = array(
						'carousel' => true,
						'margin' => empty($this->settings['carousel-item-margin'])? '': $this->settings['carousel-item-margin'],
						'overflow' => empty($this->settings['carousel-overflow'])? '': $this->settings['carousel-overflow'],
						'column' => $column_no,
						'move' => empty($this->settings['carousel-scrolling-item-amount'])? '': $this->settings['carousel-scrolling-item-amount'],
						'navigation' => empty($this->settings['carousel-navigation'])? 'navigation': $this->settings['carousel-navigation'],
						'navigation-on-hover' => empty($this->settings['carousel-navigation-show-on-hover'])? 'disable': $this->settings['carousel-navigation-show-on-hover'],
						'navigation-align' => empty($this->settings['carousel-navigation-align'])? '': $this->settings['carousel-navigation-align'],
						'navigation-size' => empty($this->settings['carousel-navigation-size'])? '': $this->settings['carousel-navigation-size'],
						'navigation-icon-color' => empty($this->settings['carousel-navigation-icon-color'])? '': $this->settings['carousel-navigation-icon-color'],
						'navigation-icon-background' => empty($this->settings['carousel-navigation-icon-bg'])? '': $this->settings['carousel-navigation-icon-bg'],
						'navigation-icon-padding' => empty($this->settings['carousel-navigation-icon-padding'])? '': $this->settings['carousel-navigation-icon-padding'],
						'navigation-icon-radius' => empty($this->settings['carousel-navigation-icon-radius'])? '': $this->settings['carousel-navigation-icon-radius'],
						'navigation-margin' => empty($this->settings['carousel-navigation-margin'])? '': $this->settings['carousel-navigation-margin'],
						'navigation-side-margin' => empty($this->settings['carousel-navigation-side-margin'])? '': $this->settings['carousel-navigation-side-margin'],
						'navigation-icon-margin' => empty($this->settings['carousel-navigation-icon-margin'])? '': $this->settings['carousel-navigation-icon-margin'],
						'navigation-left-icon' => empty($this->settings['carousel-navigation-left-icon'])? '': $this->settings['carousel-navigation-left-icon'],
						'navigation-right-icon' => empty($this->settings['carousel-navigation-right-icon'])? '': $this->settings['carousel-navigation-right-icon'],
						'bullet-style' => empty($this->settings['carousel-bullet-style'])? '': $this->settings['carousel-bullet-style'],
						'controls-top-margin' => empty($this->settings['carousel-bullet-top-margin'])? '': $this->settings['carousel-bullet-top-margin'],
						'nav-parent' => 'gdlr-core-blog-item',
						'disable-autoslide' => (empty($this->settings['carousel-autoslide']) || $this->settings['carousel-autoslide'] == 'enable')? '': true,
						'mglr' => ($this->settings['no-space'] == 'yes'? false: true)
					);
					
					gdlr_core_setup_admin_postdata();

					$blog_style = new gdlr_core_blog_style();
					while($query->have_posts()){ $query->the_post();
						$slides[] = $blog_style->get_content( $this->settings );
					} // while

					$ret .= gdlr_core_get_flexslider($slides, $flex_atts);

					wp_reset_postdata();
					gdlr_core_reset_admin_postdata();
					
				// fitrows style
				}else{

					// filterer
					if( $this->settings['filterer'] != 'none' ){
						$extra_class  = ($this->settings['no-space'] == 'yes')? '': ' gdlr-core-item-pdlr';
						$extra_class .= empty($this->settings['filterer'])? '': ' gdlr-core-style-' . $this->settings['filterer'];
						$extra_class .= empty($this->settings['filterer-align'])? '': ' gdlr-core-' . $this->settings['filterer-align'] . '-align';
						$extra_class .= apply_filters('gdlr_core_blog_filterer_class', '', $this->settings);

						$ret .= gdlr_core_get_ajax_filterer('post', 'category', $this->settings, 'gdlr-core-blog-item-holder', $extra_class);
					}
					
					gdlr_core_setup_admin_postdata();

					$ret .= '<div class="gdlr-core-blog-item-holder gdlr-core-js-2 clearfix" data-layout="' . $this->settings['layout'] . '" >';
					if( $query->have_posts() ){
						$ret .= $this->get_blog_grid_content($query);
					}
					$ret .= '</div>';

					wp_reset_postdata();
					gdlr_core_reset_admin_postdata();

					// pagination
					if( $this->settings['pagination'] != 'none' ){
						$extra_class = ($this->settings['no-space'] == 'yes')? '': 'gdlr-core-item-pdlr';

						if( $this->settings['pagination'] == 'page' ){
							if( !empty($this->settings['filterer']) && $this->settings['filterer'] != 'none' ){
								$ret .= gdlr_core_get_ajax_pagination('post', $this->settings, $query->max_num_pages, 'gdlr-core-blog-item-holder', $extra_class);
							}else{
								$ret .= gdlr_core_get_pagination($query->max_num_pages, $this->settings, $extra_class);
							}
						}else if( $this->settings['pagination'] == 'load-more' ){
							// mediaelement for blog ajax
							wp_enqueue_style('wp-mediaelement');
							wp_enqueue_script('wp-mediaelement');
							
							$paged = empty($query->query['paged'])? 2: intval($query->query['paged']) + 1;
							$ret .= gdlr_core_get_ajax_load_more('post', $this->settings, $paged, $query->max_num_pages, 'gdlr-core-blog-item-holder', $extra_class);
						}
					}
				}

				gdlr_core_set_container_multiplier(1, false);

				return $ret;
			}

			// get content of blog feature item
			function get_blog_feature_content( $query ){

				$ret = '';
				$item_mod = 0;
				$item_count = 0;
				$blog_style = new gdlr_core_blog_style();
				while($query->have_posts()){ $query->the_post();

					$item_count ++;
					$item_mod = $item_count % 10;

					$additional_class  = ' gdlr-core-column-30';
					$additional_class .= in_array($item_mod, array(1, 2, 4, 6, 8))? ' gdlr-core-column-first': '';

					if( $item_mod == 2 ){
						$ret .= '<div class="gdlr-core-item-list-wrap gdlr-core-column-30" >';
					}else if( $item_mod == 6 ){
						$ret .= '<div class="gdlr-core-item-list-wrap gdlr-core-column-30 gdlr-core-column-first" >';
					}

					$this->settings['blog-feature-main'] = in_array($item_mod, array(0, 1));
					$ret .= '<div class="gdlr-core-item-list ' . esc_attr($additional_class) . '" >';
					$ret .= $blog_style->get_content($this->settings);
					$ret .= '</div>';

					if( in_array($item_mod, array(5, 9)) ){
						$ret .= '</div>'; // gdlr-core-item-list-wrap
					}

				} // while

				if( ($item_mod >= 2 && $item_mod < 5) || ($item_mod >= 6 && $item_mod < 9) ){
					$ret .= '</div>'; // gdlr-core-item-list-wrap
				}

				return $ret;
			}

			// get content of non carousel blog item
			function get_blog_grid_content( $query ){

				if( $this->settings['blog-style'] == 'blog-widget' ){
					return $this->get_blog_grid_widget($query);
				}

				$ret = '';
				$column_sum = 0;
				$blog_style = new gdlr_core_blog_style();
				while($query->have_posts()){ $query->the_post();

					$args = $this->settings;

					if( $this->settings['has-column'] == 'yes' ){
						$additional_class  = ($this->settings['no-space'] == 'yes')? '': ' gdlr-core-item-pdlr';
						$additional_class .= in_array($this->settings['blog-style'], array('blog-image', 'blog-metro'))? ' gdlr-core-item-mgb': '';
						if( in_array($this->settings['blog-style'], array('blog-metro', 'blog-metro-no-space')) ){
							$blog_info = get_post_meta(get_the_ID(), 'gdlr-core-page-option', true);
							
							if( empty($blog_info['metro-column-size']) || $blog_info['metro-column-size'] == 'default' ){
								if( !empty($this->settings['column-size']) ){
									$additional_class .= ' gdlr-core-column-' . $this->settings['column-size'];	
								}
							}else{
								$additional_class .= ' gdlr-core-column-' . $blog_info['metro-column-size'];
							}

							if( !empty($blog_info['metro-thumbnail-size']) && $blog_info['metro-thumbnail-size'] != 'default' ){
								$args['thumbnail-size'] = $blog_info['metro-thumbnail-size'];
							}
							
						}else if( !empty($this->settings['column-size']) ){
							$additional_class .= empty($this->settings['column-size'])? '': ' gdlr-core-column-' . $this->settings['column-size'];
						}

						if( $column_sum == 0 || $column_sum + intval($this->settings['column-size']) > 60 ){
							$column_sum = intval($this->settings['column-size']);
							$additional_class .= ' gdlr-core-column-first';
						}else{
							$column_sum += intval($this->settings['column-size']);
						}

						$ret .= '<div class="gdlr-core-item-list ' . esc_attr($additional_class) . '" >';
					}

					$ret .= $blog_style->get_content($args);

					if( $this->settings['has-column'] == 'yes' ){
						$ret .= '</div>';
					}
				} // while

				return $ret;
			}
			function get_blog_grid_widget( $query ){

				$ret = apply_filters('get_blog_grid_widget', '', $query, $this);
				if( !empty($ret) ){ return $ret; }

				$first_column = true;
				$blog_widget_column = empty($this->settings['blog-widget-column'])? 60: intval($this->settings['blog-widget-column']);
				$column_item = ceil(count($query->posts) / (60 / $blog_widget_column));
				
				$count = 0;
				$featured = false;
				$blog_style = new gdlr_core_blog_style();
				while($query->have_posts()){ $query->the_post(); $count++;
					
					if( empty($featured) && !empty($this->settings['blog-widget-with-feature']) && $this->settings['blog-widget-with-feature'] == 'enable' ){
						$count = 0;
						$featured = true;
						$first_column = false;
						$column_item = 60 / $blog_widget_column;
						$column_item = ($column_item > 1)? $column_item - 1: 1;
						$column_item = ceil((count($query->posts) - 1) / $column_item);

						$temp_style = $this->settings['blog-style'];
						$this->settings['blog-style'] = 'blog-widget-feature';
						if( !empty($this->settings['blog-widget-style']) && $this->settings['blog-widget-style'] == 'style-3' && $blog_widget_column == 30 ){
							$ret .= '<div class="gdlr-core-item-list-wrap gdlr-core-featured gdlr-core-column-40" >';
							$blog_widget_column = 20;
						}else{
							$ret .= '<div class="gdlr-core-item-list-wrap gdlr-core-featured gdlr-core-column-' . esc_attr($blog_widget_column) . '" >';
						}

						$ret .= $blog_style->get_content($this->settings);

						$this->settings['blog-style'] = $temp_style;
					}else{
						if( $column_item == 1 || $count % $column_item == 1 ){
							if( $first_column ){
								$first_column = false;
							}else{
								if( !empty($this->settings['blog-widget-bottom-divider']) && $this->settings['blog-widget-bottom-divider'] == 'enable' ){
									$ret .= '<div class="gdlr-core-blog-widget-divider gdlr-core-item-mglr" ></div>';
								}
								$ret .= '</div>'; // gdlr-core-item-list-wrap
							}
							$ret .= '<div class="gdlr-core-item-list-wrap gdlr-core-column-' . esc_attr($blog_widget_column) . ' ' . ($featured? 'gdlr-core-with-featured': '') . '" >';
						}

						$ret .= $blog_style->get_content($this->settings);
					}
				} // while

				if( !empty($this->settings['blog-widget-bottom-divider']) && $this->settings['blog-widget-bottom-divider'] == 'enable' ){		
					$ret .= '<div class="gdlr-core-blog-widget-divider gdlr-core-item-mglr" ></div>';
				}
				$ret .= '</div>'; // gdlr-core-item-list-wrap

				return $ret;

			}
			
			// query the post
			function get_blog_query(){
				
				$args = array( 'post_type' => 'post', 'post_status' => 'publish', 'suppress_filters' => false );

				if( is_single() ){
					$args['post__not_in'] = array(get_the_ID());
				}
				
				// category - tag selection
				if( !empty($this->settings['category']) || !empty($this->settings['tag']) ){
					$args['tax_query'] = array(
						'relation' => empty($this->settings['relation'])? 'OR': $this->settings['relation']
					);
					
					if( !empty($this->settings['category']) ){
						$this->settings['category'] = is_array($this->settings['category'])? $this->settings['category']: explode(',', $this->settings['category']);
						array_push($args['tax_query'], array('terms'=>$this->settings['category'], 'taxonomy'=>'category', 'field'=>'slug'));
					}
					if( !empty($this->settings['tag']) ){
						$this->settings['tag'] = is_array($this->settings['tag'])? $this->settings['tag']: explode(',', $this->settings['tag']);
						array_push($args['tax_query'], array('terms'=>$this->settings['tag'], 'taxonomy'=>'post_tag', 'field'=>'slug'));
					}
				}

				// pagination
				if( $this->settings['pagination'] != 'none' ){
					if( empty($this->settings['paged']) ){
						$args['paged'] = (get_query_var('paged'))? get_query_var('paged') : get_query_var('page');
						$args['paged'] = empty($args['paged'])? 1: $args['paged'];
					}else{
						$args['paged'] = $this->settings['paged'];
					}
					$this->settings['paged'] = $args['paged'];
				}else if( !empty($this->settings['offset']) ){
					$args['offset'] = intval($this->settings['offset']);
				}
		

				// sticky post section
				if( !empty($this->settings['prepend-sticky']) && $this->settings['prepend-sticky'] == 'enable' ){
					if( empty($args['paged']) || $args['paged'] <= 1 ){
						$args['post__not_in'] = get_option('sticky_posts', '');

						$sticky_args = $args;
						$sticky_args['post__in'] = $args['post__not_in'];
						if( !empty($sticky_args['post__in']) ){
							$sticky_query = new WP_Query($sticky_args);	
						}
					}
				}else{
					$args['ignore_sticky_posts'] = 1;
				}
				
				// variable
				$args['posts_per_page'] = $this->settings['num-fetch'];
				$args['orderby'] = $this->settings['orderby'];
				$args['order'] = $this->settings['order'];
				
				$query = new WP_Query( $args );

				// merge query
				if( !empty($sticky_query) ){
					$query->posts = array_merge($sticky_query->posts, $query->posts);
					$query->post_count = $sticky_query->post_count + $query->post_count;
				}

				return $query;
			}
			
		} // gdlr_core_blog_item
	} // class_exists
	
	add_action('wp_ajax_gdlr_core_post_ajax', 'gdlr_core_post_ajax');
	add_action('wp_ajax_nopriv_gdlr_core_post_ajax', 'gdlr_core_post_ajax');
	if( !function_exists('gdlr_core_post_ajax') ){
		function gdlr_core_post_ajax(){

			if( !empty($_POST['settings']) ){

				$settings = $_POST['settings'];
				if( !empty($_POST['option']['name']) && !empty($_POST['option']['value']) ){	
					if( in_array($_POST['option']['name'], array('paged', 'category')) ){ 
						$settings[$_POST['option']['name']] = $_POST['option']['value'];

						if( $_POST['option']['name'] == 'category' ){
							$settings['paged'] = 1;
							// unset($settings['tag']);
						}
					}
				}else{
					$settings['paged'] = 1;
				}

				$blog_item = new gdlr_core_blog_item($settings);
				$query = $blog_item->get_blog_query();	

				$ret = array(
					'status'=> 'success',
					'content'=> $blog_item->get_blog_grid_content($query)
				);
				if( !empty($settings['pagination']) && $settings['pagination'] != 'none' ){
					$extra_class = ($settings['no-space'] == 'yes')? '': 'gdlr-core-item-pdlr';

					if( $settings['pagination'] == 'load-more' ){
						$paged = empty($query->query['paged'])? 2: intval($query->query['paged']) + 1;

						$ret['load_more'] = gdlr_core_get_ajax_load_more('post', $settings, $paged, $query->max_num_pages, 'gdlr-core-blog-item-holder', $extra_class);
						$ret['load_more'] = empty($ret['load_more'])? 'none': $ret['load_more'];
					
					// change pagination on category filter
					}else if( empty($_POST['option']['name']) || $_POST['option']['name'] == 'category' ){
						$ret['pagination'] = gdlr_core_get_ajax_pagination('post', $settings, $query->max_num_pages, 'gdlr-core-blog-item-holder', $extra_class);
						$ret['pagination'] = empty($ret['pagination'])? 'none': $ret['pagination'];
					}
				} 

				die(json_encode($ret));
			}else{
				die(json_encode(array(
					'status'=> 'failed',
					'message'=> esc_html__('Settings variable is not defined.', 'goodlayers-core')
				)));
			}

		} // gdlr_core_post_load_more
	} // function_exists
	
	
	
	
	
	
	
	
	
	
	
	
	
	