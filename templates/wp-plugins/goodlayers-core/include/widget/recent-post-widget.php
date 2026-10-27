<?php
	/**
	 * A widget that show recent posts ( Specified by category ).
	 */

	add_action('widgets_init', 'gdlr_core_recent_post_widget');
	if( !function_exists('gdlr_core_recent_post_widget') ){
		function gdlr_core_recent_post_widget() {
			register_widget( 'Goodlayers_Core_Recent_Post_Widget' );
		}
	}

	if( !class_exists('Goodlayers_Core_Recent_Post_Widget') ){
		class Goodlayers_Core_Recent_Post_Widget extends WP_Widget{

			// Initialize the widget
			function __construct() {

				parent::__construct(
					'gdlr-core-recent-post-widget', 
					esc_html__('Recent Post Widget ( Goodlayers )', 'goodlayers-core'), 
					array('description' => esc_html__('A widget that show latest posts', 'goodlayers-core'))
				);  

			}

			// Output of the widget
			function widget( $args, $instance ) {
	
				$title = empty($instance['title'])? '': apply_filters('widget_title', $instance['title']);
				$style = empty($instance['style'])? 'style-1': $instance['style'];
				$category = empty($instance['category'])? '': $instance['category'];
				$num_fetch = empty($instance['num-fetch'])? '': $instance['num-fetch'];
				$thumbnail_size = empty($instance['thumbnail-size'])? 'thumbnail': $instance['thumbnail-size'];
				$orderby = empty($instance['order-by'])? 'thumbnail': $instance['order-by'];
				$order = empty($instance['order'])? 'thumbnail': $instance['order'];
				
				// Opening of widget
				echo gdlr_core_escape_content($args['before_widget']);
				
				// Open of title tag
				if( !empty($title) ){ 
					echo gdlr_core_escape_content($args['before_title'] . $title . $args['after_title']); 
				}
					
				// Widget Content
				$query_args = array(
					'post_type' => 'post', 
					'suppress_filters' => false,
					'orderby' => $orderby,
					'order' => $order,
					'paged' => 1,
					'ignore_sticky_posts' => 1,
					'posts_per_page' => $num_fetch,
					'category_name' => $category,
					'post__not_in' => array(get_the_ID())
				);
				$query = new WP_Query( $query_args );
				$blog_style = apply_filters('gdlr_core_recent_post_widget_blog_style', '');
				if( empty($blog_style) ){
					$blog_style = new gdlr_core_blog_style();
				}
				
				
				if($query->have_posts()){
					echo '<div class="gdlr-core-recent-post-widget-wrap gdlr-core-' . esc_attr($style) . '">';
					
					gdlr_core_setup_admin_postdata();
					while($query->have_posts()){ $query->the_post();
						$feature_image = get_post_thumbnail_id();

						echo '<div class="gdlr-core-recent-post-widget clearfix">';
						if( !empty($feature_image) ){
							if( in_array($style, array('style-1','style-3','style-4','style-6')) ){
								echo '<div class="gdlr-core-recent-post-widget-thumbnail gdlr-core-media-image" >';
								echo '<a href="' . esc_attr(get_permalink()) . '" >';
								echo gdlr_core_get_image($feature_image, $thumbnail_size);
								echo '</a>';
								echo '</div>';
							}else if( $style == 'style-full' ){
								echo '<div class="gdlr-core-recent-post-widget-thumbnail gdlr-core-media-image" >';
								if( get_post_format() == 'video' ){
									global $pages;
									if( !preg_match('#^https?://\S+#', $pages[0], $match) ){
										if( !preg_match('#^\[video\s.+\[/video\]#', $pages[0], $match) ){
											preg_match('#^\[embed.+\[/embed\]#', $pages[0], $match);
										}
									}

									if( !empty($match[0]) ){
										echo '<a ' . gdlr_core_get_lightbox_atts(array(
											'type' => 'video', 
											'url' => $match[0]
										)) . ' >';
										echo '<span class="gdlr-core-recent-post-widget-thumbnail-video" ><i class="fa fa-play" ></i></span>';
									}else{
										echo '<a ' . gdlr_core_get_lightbox_atts(array(
											'type' => 'image', 
											'url' => gdlr_core_get_image_url($feature_image)
										)) . ' >';
									}
								}else{
									echo '<a ' . gdlr_core_get_lightbox_atts(array(
										'type' => 'image', 
										'url' => gdlr_core_get_image_url($feature_image)
									)) . ' >';
								}
								echo gdlr_core_get_image($feature_image, $thumbnail_size);
								echo '</a>'; // post format link

								echo $blog_style->blog_info(array(
									'display' => array('category'),
								));
								echo '</div>';
							}
						}

						echo '<div class="gdlr-core-recent-post-widget-content">';
						if( $style == 'style-1' ){	
							$blog_info = apply_filters('gdlr_core_recent_post_widget_blog_info', array('date', 'author'), $style);
							echo '<div class="gdlr-core-recent-post-widget-title"><a href="' . get_permalink() . '" >' . get_the_title() . '</a></div>';
							echo '<div class="gdlr-core-recent-post-widget-info">' . $blog_style->blog_info(array('display' => $blog_info)) . '</div>';
						}else if( $style == 'style-2' ){
							$blog_info = apply_filters('gdlr_core_recent_post_widget_blog_info', array('date'), $style);
							echo '<div class="gdlr-core-recent-post-widget-title"><a href="' . get_permalink() . '" >' . get_the_title() . '</a></div>';
							echo '<div class="gdlr-core-recent-post-widget-info">' . $blog_style->blog_info(array('display' => $blog_info)) . '</div>';
						}else if( $style == 'style-5' ){
							$blog_info = apply_filters('gdlr_core_recent_post_widget_blog_info', array('date'), $style);
							echo '<div class="gdlr-core-recent-post-widget-info">' . $blog_style->blog_info(array('display' => $blog_info)) . '</div>';
							echo '<div class="gdlr-core-recent-post-widget-title"><a href="' . get_permalink() . '" >' . get_the_title() . '</a></div>';
						}else if( $style == 'style-6' ){
							$blog_info = apply_filters('gdlr_core_recent_post_widget_blog_info', array('date', 'category'), $style);
							echo '<div class="gdlr-core-recent-post-widget-title"><a href="' . get_permalink() . '" >' . get_the_title() . '</a></div>';
							echo '<div class="gdlr-core-recent-post-widget-info">' . $blog_style->blog_info(array('display' => $blog_info, 'separator' => '•', 'icon' => false)) . '</div>';
						}else if( $style == 'style-3' ){
							$prefix = $blog_style->get_blog_info_prefix();
							$prefix['date'] = '<i class="fa fa-clock-o" ></i>';
							$blog_style->set_blog_info_prefix($prefix);
							$blog_info = apply_filters('gdlr_core_recent_post_widget_blog_info', array('date'), $style);

							echo '<div class="gdlr-core-recent-post-widget-title gdlr-core-title-font"><a href="' . get_permalink() . '" >' . get_the_title() . '</a></div>';
							echo '<div class="gdlr-core-recent-post-widget-info">' . $blog_style->blog_info(array('display' => $blog_info)) . '</div>';
						}else if( $style == 'style-4' ){	
							$blog_info = apply_filters('gdlr_core_recent_post_widget_blog_info', array('date'), $style);
							echo '<div class="gdlr-core-recent-post-widget-info">' . $blog_style->blog_info(array('display' => $blog_info)) . '</div>';
							echo '<div class="gdlr-core-recent-post-widget-title gdlr-core-title-font"><a href="' . get_permalink() . '" >' . get_the_title() . '</a></div>';
						}else if( $style == 'style-full' ){
							$blog_info = apply_filters('gdlr_core_recent_post_widget_blog_info', array('date', 'comment-number'), $style);

							echo '<div class="gdlr-core-recent-post-widget-title gdlr-core-title-font"><a href="' . get_permalink() . '" >' . get_the_title() . '</a></div>';
							echo '<div class="gdlr-core-recent-post-widget-info">' . $blog_style->blog_info(array('display' => $blog_info)) . '</div>';
						}
						echo '</div>'; // gdlr-core-recent-post-widget-content
						echo '</div>'; // gdlr-core-recent-post-widget

					}
					wp_reset_postdata();
					gdlr_core_reset_admin_postdata();
					echo '</div>'; // gdlr-core-recent-post-widget-wrap
				}
				
				// Closing of widget
				echo gdlr_core_escape_content($args['after_widget']);

			}

			// Widget Form
			function form( $instance ) {

				if( class_exists('gdlr_core_widget_util') ){
					gdlr_core_widget_util::get_option(array(
						'title' => array(
							'type' => 'text',
							'id' => $this->get_field_id('title'),
							'name' => $this->get_field_name('title'),
							'title' => esc_html__('Title', 'goodlayers-core'),
							'value' => (isset($instance['title'])? $instance['title']: '')
						),
						'style' => array(
							'type' => 'combobox',
							'id' => $this->get_field_id('style'),
							'name' => $this->get_field_name('style'),
							'title' => esc_html__('Style', 'goodlayers-core'),
							'options' => array(
								'style-1' => esc_html__('Normal', 'goodlayers-core'),
								'style-3' => esc_html__('Normal 2', 'goodlayers-core'),
								'style-4' => esc_html__('Normal 3', 'goodlayers-core'),
								'style-6' => esc_html__('Normal 4 (dot info)', 'goodlayers-core'),
								'style-2' => esc_html__('Short', 'goodlayers-core'),
								'style-5' => esc_html__('Short 2', 'goodlayers-core'),
								'style-full' => esc_html__('Full', 'goodlayers-core'),
							),
							'value' => (isset($instance['category'])? $instance['style']: '')
						),
						'category' => array(
							'type' => 'combobox',
							'id' => $this->get_field_id('category'),
							'name' => $this->get_field_name('category'),
							'title' => esc_html__('Category', 'goodlayers-core'),
							'options' => gdlr_core_get_term_list('category', '', true),
							'value' => (isset($instance['category'])? $instance['category']: '')
						),
						'num-fetch' => array(
							'type' => 'text',
							'id' => $this->get_field_id('num-fetch'),
							'name' => $this->get_field_name('num-fetch'),
							'title' => esc_html__('Display Number', 'goodlayers-core'), 
							'value' => (isset($instance['num-fetch'])? $instance['num-fetch']: '3')
						),
						'order-by' => array(
							'type' => 'combobox',
							'id' => $this->get_field_id('order-by'),
							'name' => $this->get_field_name('order-by'),
							'title' => esc_html__('Order By', 'goodlayers-core'), 
							'options' => array(
								'date' => esc_html__('Publish Date', 'goodlayers-core'), 
								'title' => esc_html__('Title', 'goodlayers-core'), 
								'rand' => esc_html__('Random', 'goodlayers-core'), 
								'menu_order' => esc_html__('Menu Order', 'goodlayers-core'), 
								'comment_count' => esc_html__('Comments Count', 'goodlayers-core'), 
							),
							'value' => (isset($instance['order-by'])? $instance['order-by']: 'date')
						),
						'order' => array(
							'type' => 'combobox',
							'id' => $this->get_field_id('order'),
							'name' => $this->get_field_name('order'),
							'title' => esc_html__('Order', 'goodlayers-core'), 
							'options' => array(
								'desc'=>esc_html__('Descending Order', 'goodlayers-core'), 
								'asc'=> esc_html__('Ascending Order', 'goodlayers-core'), 
							),
							'value' => (isset($instance['order'])? $instance['order']: 'desc')
						),
						'thumbnail-size' => array(
							'type' => 'combobox',
							'id' => $this->get_field_id('thumbnail-size'),
							'name' => $this->get_field_name('thumbnail-size'),
							'title' => esc_html__('Thumbnail Size', 'goodlayers-core'), 
							'options' => gdlr_core_get_thumbnail_list(),
							'value' => (isset($instance['thumbnail-size'])? $instance['thumbnail-size']: 'thumbnail')
						),
					));
				}

			}
			
			// Update the widget
			function update( $new_instance, $old_instance ) {

				if( class_exists('gdlr_core_widget_util') ){
					return gdlr_core_widget_util::get_option_update($new_instance);
				}

				return $new_instance;
			}	
		} // class
	} // class_exists
?>